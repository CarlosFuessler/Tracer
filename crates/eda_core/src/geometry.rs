//! 2D geometry primitives for schematic editing.

/// A point in schematic space (millimeters).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Snap to the nearest grid point.
    #[must_use]
    pub fn snapped(self, grid: f64) -> Self {
        if grid <= 0.0 {
            return self;
        }
        Self {
            x: (self.x / grid).round() * grid,
            y: (self.y / grid).round() * grid,
        }
    }

    /// Distance to another point.
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Offset by a delta.
    #[must_use]
    pub fn offset(self, dx: f64, dy: f64) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl std::fmt::Display for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

/// A wire segment between two points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WireSegment {
    pub start: Point2D,
    pub end: Point2D,
}

impl WireSegment {
    #[must_use]
    pub const fn new(start: Point2D, end: Point2D) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn length(&self) -> f64 {
        self.start.distance_to(self.end)
    }

    /// Snap both endpoints to grid.
    #[must_use]
    pub fn snapped(self, grid: f64) -> Self {
        Self {
            start: self.start.snapped(grid),
            end: self.end.snapped(grid),
        }
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoundingBox {
    pub min: Point2D,
    pub max: Point2D,
}

impl BoundingBox {
    #[must_use]
    pub const fn new(min: Point2D, max: Point2D) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub fn contains(&self, point: Point2D) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    #[must_use]
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    #[must_use]
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    #[must_use]
    pub fn center(&self) -> Point2D {
        Point2D::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }

    /// Create a bounding box around a point with given half-size.
    #[must_use]
    pub fn around(center: Point2D, half_w: f64, half_h: f64) -> Self {
        Self {
            min: Point2D::new(center.x - half_w, center.y - half_h),
            max: Point2D::new(center.x + half_w, center.y + half_h),
        }
    }
}

// ── Symbol graphics primitives ─────────────────────────

/// Direction a pin extends from its connection point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PinDirection {
    #[default]
    Right,
    Left,
    Up,
    Down,
}

impl PinDirection {
    /// Create from KiCad angle (degrees: 0=right, 90=up, 180=left, 270=down).
    #[must_use]
    pub fn from_kicad_angle(deg: f64) -> Self {
        let norm = ((deg % 360.0) + 360.0) % 360.0;
        if norm < 45.0 || norm >= 315.0 {
            Self::Right
        } else if norm < 135.0 {
            Self::Up
        } else if norm < 225.0 {
            Self::Left
        } else {
            Self::Down
        }
    }

    /// Unit vector in schematic space (KiCad Y-down convention).
    #[must_use]
    pub fn unit(self) -> (f64, f64) {
        match self {
            Self::Right => (1.0, 0.0),
            Self::Left => (-1.0, 0.0),
            Self::Up => (0.0, -1.0),
            Self::Down => (0.0, 1.0),
        }
    }
}

/// A pin on a schematic symbol.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SymbolPin {
    pub name: String,
    pub number: String,
    /// Connection point in symbol-local coordinates (mm).
    pub position: Point2D,
    /// Direction the pin wire extends.
    pub direction: PinDirection,
    /// Pin length in mm.
    pub length: f64,
}

impl SymbolPin {
    /// The endpoint of the pin stub (opposite of the connection point).
    #[must_use]
    pub fn stub_end(&self) -> Point2D {
        let (dx, dy) = self.direction.unit();
        self.position.offset(-dx * self.length, -dy * self.length)
    }
}

/// A rectangle in symbol-local coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SymbolRect {
    pub start: Point2D,
    pub end: Point2D,
}

impl SymbolRect {
    #[must_use]
    pub fn width(&self) -> f64 {
        (self.end.x - self.start.x).abs()
    }

    #[must_use]
    pub fn height(&self) -> f64 {
        (self.end.y - self.start.y).abs()
    }

    #[must_use]
    pub fn top_left(&self) -> Point2D {
        Point2D::new(self.start.x.min(self.end.x), self.start.y.min(self.end.y))
    }
}

/// A polyline segment in symbol-local coordinates.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SymbolPolyline {
    pub points: Vec<Point2D>,
}

/// A circle in symbol-local coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SymbolCircle {
    pub center: Point2D,
    pub radius: f64,
}

/// Complete graphics description for a schematic symbol.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SymbolGraphics {
    pub pins: Vec<SymbolPin>,
    pub rectangles: Vec<SymbolRect>,
    pub polylines: Vec<SymbolPolyline>,
    pub circles: Vec<SymbolCircle>,
    /// Reference designator prefix (e.g. "R", "C", "U").
    pub reference: String,
    /// Value / description text.
    pub value: String,
}

impl SymbolGraphics {
    /// Compute the bounding box of the symbol body (excluding pins).
    #[must_use]
    pub fn body_bounds(&self) -> BoundingBox {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for r in &self.rectangles {
            min_x = min_x.min(r.start.x).min(r.end.x);
            min_y = min_y.min(r.start.y).min(r.end.y);
            max_x = max_x.max(r.start.x).max(r.end.x);
            max_y = max_y.max(r.start.y).max(r.end.y);
        }
        for pl in &self.polylines {
            for p in &pl.points {
                min_x = min_x.min(p.x);
                min_y = min_y.min(p.y);
                max_x = max_x.max(p.x);
                max_y = max_y.max(p.y);
            }
        }
        for c in &self.circles {
            min_x = min_x.min(c.center.x - c.radius);
            min_y = min_y.min(c.center.y - c.radius);
            max_x = max_x.max(c.center.x + c.radius);
            max_y = max_y.max(c.center.y + c.radius);
        }

        if min_x > max_x {
            // No geometry — give a default size
            return BoundingBox::around(Point2D::zero(), 2.54, 2.54);
        }
        BoundingBox::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_snaps_to_grid() {
        let p = Point2D::new(1.3, 2.7);
        let snapped = p.snapped(1.27);
        assert!((snapped.x - 1.27).abs() < 0.001);
        assert!((snapped.y - 2.54).abs() < 0.001);
    }

    #[test]
    fn bounding_box_contains_point() {
        let bb = BoundingBox::new(Point2D::new(0.0, 0.0), Point2D::new(10.0, 10.0));
        assert!(bb.contains(Point2D::new(5.0, 5.0)));
        assert!(!bb.contains(Point2D::new(11.0, 5.0)));
    }

    #[test]
    fn wire_segment_length() {
        let w = WireSegment::new(Point2D::new(0.0, 0.0), Point2D::new(3.0, 4.0));
        assert!((w.length() - 5.0).abs() < 0.001);
    }
}
