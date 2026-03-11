//! Extract symbol graphics from KiCad `.kicad_sym` files using kiutils_kicad.
//!
//! Walks the lossless CST to extract pin positions, body rectangles,
//! polylines and circles — everything needed to render a schematic symbol.

use std::collections::HashMap;
use std::path::Path;

use eda_core::{
    PinDirection, Point2D, SymbolCircle, SymbolGraphics, SymbolPin, SymbolPolyline, SymbolRect,
};
use kiutils_kicad::SymbolLibFile;
use kiutils_sexpr::{Atom, Node};

/// Parsed symbol library: maps symbol name → graphics.
#[derive(Debug, Clone, Default)]
pub struct ParsedSymbolLib {
    pub symbols: HashMap<String, SymbolGraphics>,
}

/// List all top-level symbol names in a `.kicad_sym` file.
pub fn list_symbol_names(path: &Path) -> Vec<String> {
    let doc = match SymbolLibFile::read(path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    doc.ast()
        .symbols
        .iter()
        .filter_map(|s| s.name.clone())
        .collect()
}

/// Parse all symbols from a `.kicad_sym` file, returning a map of name → graphics.
pub fn parse_symbol_lib(path: &Path) -> ParsedSymbolLib {
    let doc = match SymbolLibFile::read(path) {
        Ok(d) => d,
        Err(_) => return ParsedSymbolLib::default(),
    };

    let cst = doc.cst();
    let mut lib = ParsedSymbolLib::default();

    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return lib;
    };

    for item in items.iter().skip(1) {
        if head_of(item) == Some("symbol") {
            if let Some((name, graphics)) = extract_symbol(item) {
                lib.symbols.insert(name, graphics);
            }
        }
    }

    lib
}

/// Parse a single named symbol from a `.kicad_sym` file.
pub fn parse_one_symbol(path: &Path, symbol_name: &str) -> Option<SymbolGraphics> {
    let doc = SymbolLibFile::read(path).ok()?;
    let cst = doc.cst();
    let Node::List { items, .. } = cst.nodes.first()? else {
        return None;
    };

    for item in items.iter().skip(1) {
        if head_of(item) == Some("symbol") {
            if let Some((name, graphics)) = extract_symbol(item) {
                if name == symbol_name {
                    return Some(graphics);
                }
            }
        }
    }
    None
}

// ── CST walking ─────────────────────────────────────────

fn extract_symbol(node: &Node) -> Option<(String, SymbolGraphics)> {
    let Node::List { items, .. } = node else {
        return None;
    };
    let name = atom_string(items.get(1)?)?;
    let mut gfx = SymbolGraphics::default();

    // Extract properties at the top-level of this symbol
    for child in items.iter().skip(2) {
        match head_of(child) {
            Some("property") => extract_property(child, &mut gfx),
            Some("symbol") => {
                // Sub-symbol units contain the actual graphics
                extract_subsymbol_graphics(child, &mut gfx);
            }
            Some("pin") => {
                if let Some(pin) = extract_pin(child) {
                    gfx.pins.push(pin);
                }
            }
            Some("rectangle") => {
                if let Some(r) = extract_rectangle(child) {
                    gfx.rectangles.push(r);
                }
            }
            Some("polyline") => {
                if let Some(pl) = extract_polyline(child) {
                    gfx.polylines.push(pl);
                }
            }
            Some("circle") => {
                if let Some(c) = extract_circle(child) {
                    gfx.circles.push(c);
                }
            }
            _ => {}
        }
    }

    Some((name, gfx))
}

fn extract_subsymbol_graphics(node: &Node, gfx: &mut SymbolGraphics) {
    let Node::List { items, .. } = node else {
        return;
    };
    for child in items.iter().skip(2) {
        match head_of(child) {
            Some("pin") => {
                if let Some(pin) = extract_pin(child) {
                    gfx.pins.push(pin);
                }
            }
            Some("rectangle") => {
                if let Some(r) = extract_rectangle(child) {
                    gfx.rectangles.push(r);
                }
            }
            Some("polyline") => {
                if let Some(pl) = extract_polyline(child) {
                    gfx.polylines.push(pl);
                }
            }
            Some("circle") => {
                if let Some(c) = extract_circle(child) {
                    gfx.circles.push(c);
                }
            }
            _ => {}
        }
    }
}

fn extract_property(node: &Node, gfx: &mut SymbolGraphics) {
    let Node::List { items, .. } = node else {
        return;
    };
    // (property "Reference" "R" ...)
    let key = items.get(1).and_then(atom_string);
    let val = items.get(2).and_then(atom_string);
    match key.as_deref() {
        Some("Reference") => {
            if let Some(v) = val {
                gfx.reference = v;
            }
        }
        Some("Value") => {
            if let Some(v) = val {
                gfx.value = v;
            }
        }
        _ => {}
    }
}

fn extract_pin(node: &Node) -> Option<SymbolPin> {
    // (pin <type> <shape> (at x y [angle]) (length l) (name "n" ...) (number "n" ...))
    let Node::List { items, .. } = node else {
        return None;
    };

    let mut pin = SymbolPin::default();

    for child in items.iter().skip(1) {
        match head_of(child) {
            Some("at") => {
                let vals = list_floats(child);
                if vals.len() >= 2 {
                    pin.position = Point2D::new(vals[0], vals[1]);
                }
                if vals.len() >= 3 {
                    pin.direction = PinDirection::from_kicad_angle(vals[2]);
                }
            }
            Some("length") => {
                let vals = list_floats(child);
                if let Some(&l) = vals.first() {
                    pin.length = l;
                }
            }
            Some("name") => {
                if let Node::List { items: name_items, .. } = child {
                    pin.name = name_items.get(1).and_then(atom_string).unwrap_or_default();
                }
            }
            Some("number") => {
                if let Node::List { items: num_items, .. } = child {
                    pin.number = num_items.get(1).and_then(atom_string).unwrap_or_default();
                }
            }
            _ => {}
        }
    }

    Some(pin)
}

fn extract_rectangle(node: &Node) -> Option<SymbolRect> {
    // (rectangle (start x y) (end x y) ...)
    let Node::List { items, .. } = node else {
        return None;
    };

    let mut start = Point2D::zero();
    let mut end = Point2D::zero();

    for child in items.iter().skip(1) {
        match head_of(child) {
            Some("start") => {
                let vals = list_floats(child);
                if vals.len() >= 2 {
                    start = Point2D::new(vals[0], vals[1]);
                }
            }
            Some("end") => {
                let vals = list_floats(child);
                if vals.len() >= 2 {
                    end = Point2D::new(vals[0], vals[1]);
                }
            }
            _ => {}
        }
    }

    Some(SymbolRect { start, end })
}

fn extract_polyline(node: &Node) -> Option<SymbolPolyline> {
    // (polyline (pts (xy x y) (xy x y) ...) ...)
    let Node::List { items, .. } = node else {
        return None;
    };

    let mut points = Vec::new();

    for child in items.iter().skip(1) {
        if head_of(child) == Some("pts") {
            if let Node::List { items: pts_items, .. } = child {
                for pt in pts_items.iter().skip(1) {
                    if head_of(pt) == Some("xy") {
                        let vals = list_floats(pt);
                        if vals.len() >= 2 {
                            points.push(Point2D::new(vals[0], vals[1]));
                        }
                    }
                }
            }
        }
    }

    if points.len() < 2 {
        return None;
    }

    Some(SymbolPolyline { points })
}

fn extract_circle(node: &Node) -> Option<SymbolCircle> {
    // (circle (center x y) (radius r) ...)
    let Node::List { items, .. } = node else {
        return None;
    };

    let mut center = Point2D::zero();
    let mut radius = 0.0;

    for child in items.iter().skip(1) {
        match head_of(child) {
            Some("center") => {
                let vals = list_floats(child);
                if vals.len() >= 2 {
                    center = Point2D::new(vals[0], vals[1]);
                }
            }
            Some("radius") => {
                let vals = list_floats(child);
                if let Some(&r) = vals.first() {
                    radius = r;
                }
            }
            _ => {}
        }
    }

    if radius <= 0.0 {
        return None;
    }

    Some(SymbolCircle { center, radius })
}

// ── CST utilities ───────────────────────────────────────

fn head_of(node: &Node) -> Option<&str> {
    match node {
        Node::List { items, .. } => match items.first() {
            Some(Node::Atom { atom, .. }) => Some(atom_str(atom)),
            _ => None,
        },
        _ => None,
    }
}

fn atom_string(node: &Node) -> Option<String> {
    match node {
        Node::Atom { atom, .. } => Some(atom_str(atom).to_owned()),
        _ => None,
    }
}

fn atom_str(atom: &Atom) -> &str {
    match atom {
        Atom::Symbol(s) | Atom::Quoted(s) => s.as_str(),
    }
}

/// Extract all numeric atoms from a list node (skipping the head).
fn list_floats(node: &Node) -> Vec<f64> {
    let Node::List { items, .. } = node else {
        return Vec::new();
    };
    items
        .iter()
        .skip(1)
        .filter_map(|n| {
            if let Node::Atom { atom, .. } = n {
                atom_str(atom).parse::<f64>().ok()
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fixture_symbol_lib() {
        let path = Path::new("../../fixtures/kicad/library/basic.kicad_sym");
        if !path.exists() {
            return; // skip if fixtures not available
        }
        let lib = parse_symbol_lib(path);
        assert!(lib.symbols.contains_key("Device:R"), "should find Device:R");
        let r = &lib.symbols["Device:R"];
        assert_eq!(r.pins.len(), 2, "resistor has 2 pins");
        assert_eq!(r.rectangles.len(), 1, "resistor has 1 rectangle body");
    }

    #[test]
    fn list_names_from_fixture() {
        let path = Path::new("../../fixtures/kicad/library/basic.kicad_sym");
        if !path.exists() {
            return;
        }
        let names = list_symbol_names(path);
        assert!(names.contains(&"Device:R".to_string()));
    }

    #[test]
    fn parse_system_device_lib() {
        let path = Path::new(
            "/Applications/KiCad/KiCad.app/Contents/SharedSupport/symbols/Device.kicad_sym",
        );
        if !path.exists() {
            return; // skip if KiCad not installed
        }
        let lib = parse_symbol_lib(path);
        assert!(!lib.symbols.is_empty(), "should find symbols");

        if let Some(r) = lib.symbols.get("R") {
            assert!(!r.pins.is_empty(), "resistor should have pins");
            assert!(!r.rectangles.is_empty(), "resistor should have body");
        }
    }
}
