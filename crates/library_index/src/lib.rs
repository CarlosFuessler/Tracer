#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryKind {
    Symbol,
    Footprint,
    ThreeDimensionalModel,
}

impl LibraryKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Symbol => "symbol",
            Self::Footprint => "footprint",
            Self::ThreeDimensionalModel => "3d-model",
        }
    }

    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Symbol => "kicad_sym",
            Self::Footprint => "pretty",
            Self::ThreeDimensionalModel => "3dshapes",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibrarySource {
    name: String,
    path: PathBuf,
    kind: LibraryKind,
    /// For individual symbols: the symbol name inside the .kicad_sym file.
    /// Empty for library-level entries (footprints, 3D models).
    symbol_name: String,
}

impl LibrarySource {
    #[must_use]
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>, kind: LibraryKind) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            kind,
            symbol_name: String::new(),
        }
    }

    /// Create a source for an individual symbol within a .kicad_sym file.
    #[must_use]
    pub fn symbol(
        display_name: impl Into<String>,
        path: impl Into<PathBuf>,
        symbol_name: impl Into<String>,
    ) -> Self {
        Self {
            name: display_name.into(),
            path: path.into(),
            kind: LibraryKind::Symbol,
            symbol_name: symbol_name.into(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub const fn kind(&self) -> LibraryKind {
        self.kind
    }

    /// The symbol name inside the .kicad_sym file (empty for non-symbol entries).
    #[must_use]
    pub fn symbol_name(&self) -> &str {
        &self.symbol_name
    }

    /// Load the actual symbol graphics for this source (if it's a symbol).
    #[must_use]
    pub fn load_graphics(&self) -> Option<eda_core::SymbolGraphics> {
        if self.kind != LibraryKind::Symbol || self.symbol_name.is_empty() {
            return None;
        }
        kicad_fmt::symbol_parser::parse_one_symbol(&self.path, &self.symbol_name)
    }
}

#[derive(Debug, Default, Clone)]
pub struct LibraryCatalog {
    sources: Vec<LibrarySource>,
    /// Tracks which .kicad_sym files have already been expanded into individual symbols.
    expanded_libs: HashSet<PathBuf>,
}

impl LibraryCatalog {
    pub fn add_source(&mut self, source: LibrarySource) {
        if !self.sources.iter().any(|s| {
            s.path == source.path && s.symbol_name == source.symbol_name
        }) {
            self.sources.push(source);
        }
    }

    #[must_use]
    pub fn sources(&self) -> &[LibrarySource] {
        &self.sources
    }

    #[must_use]
    pub fn by_kind(&self, kind: LibraryKind) -> Vec<&LibrarySource> {
        self.sources.iter().filter(|s| s.kind == kind).collect()
    }

    /// Expand a library-level .kicad_sym entry into individual symbol entries.
    /// This parses the file once and replaces the library-level entry.
    fn expand_symbol_library(&mut self, lib_path: &Path) {
        if self.expanded_libs.contains(lib_path) {
            return;
        }
        self.expanded_libs.insert(lib_path.to_path_buf());

        let lib_name = lib_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        let symbol_names = kicad_fmt::symbol_parser::list_symbol_names(lib_path);
        if symbol_names.is_empty() {
            return;
        }

        // Remove the library-level placeholder entry
        self.sources.retain(|s| {
            !(s.path == lib_path && s.symbol_name.is_empty() && s.kind == LibraryKind::Symbol)
        });

        // Add individual symbol entries
        for sym_name in symbol_names {
            let display = format!("{}:{}", lib_name, sym_name);
            self.add_source(LibrarySource::symbol(&display, lib_path, &sym_name));
        }
    }

    /// Lazily expand all symbol libraries that match the search query, then search.
    #[must_use]
    pub fn search(&mut self, query: &str) -> Vec<&LibrarySource> {
        let query_lower = query.trim().to_ascii_lowercase();

        if query_lower.is_empty() {
            return self.sources.iter().collect();
        }

        // Find unexpanded symbol library files matching the query
        let libs_to_expand: Vec<PathBuf> = self
            .sources
            .iter()
            .filter(|s| {
                s.kind == LibraryKind::Symbol
                    && s.symbol_name.is_empty()
                    && !self.expanded_libs.contains(&s.path)
                    && (s.name.to_ascii_lowercase().contains(&query_lower)
                        || s.path.to_string_lossy().to_ascii_lowercase().contains(&query_lower))
            })
            .map(|s| s.path.clone())
            .collect();

        for lib_path in libs_to_expand {
            self.expand_symbol_library(&lib_path);
        }

        self.sources
            .iter()
            .filter(|source| {
                source.name.to_ascii_lowercase().contains(&query_lower)
                    || source
                        .path
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .contains(&query_lower)
            })
            .collect()
    }
}

/// Auto-detect KiCad library locations on the current system.
/// Returns a catalog pre-populated with every symbol library, footprint library,
/// and 3D model directory found in standard installation paths.
#[must_use]
pub fn detect_system_libraries() -> LibraryCatalog {
    let mut catalog = LibraryCatalog::default();

    for search_root in system_library_roots() {
        if !search_root.is_dir() {
            continue;
        }
        scan_library_dir(
            &search_root,
            "symbols",
            "kicad_sym",
            LibraryKind::Symbol,
            &mut catalog,
        );
        scan_library_dir(
            &search_root,
            "footprints",
            "pretty",
            LibraryKind::Footprint,
            &mut catalog,
        );
        scan_3d_dir(&search_root, &mut catalog);
    }

    catalog
}

fn system_library_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "macos")]
    {
        roots.push(PathBuf::from(
            "/Applications/KiCad/KiCad.app/Contents/SharedSupport",
        ));
        if let Some(home) = home_dir() {
            roots.push(home.join("Library/Application Support/kicad"));
        }
        roots.push(PathBuf::from("/Library/Application Support/kicad"));
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        roots.push(PathBuf::from("/usr/share/kicad"));
        roots.push(PathBuf::from("/usr/local/share/kicad"));
        if let Some(home) = home_dir() {
            roots.push(home.join(".local/share/kicad"));
            roots.push(home.join(".kicad"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        roots.push(PathBuf::from("C:\\Program Files\\KiCad\\share\\kicad"));
        roots.push(PathBuf::from(
            "C:\\Program Files (x86)\\KiCad\\share\\kicad",
        ));
        if let Ok(appdata) = std::env::var("APPDATA") {
            roots.push(PathBuf::from(appdata).join("kicad"));
        }
    }

    // Also check KICAD env vars
    for var in [
        "KICAD_SYMBOL_DIR",
        "KICAD8_SYMBOL_DIR",
        "KICAD7_SYMBOL_DIR",
        "KICAD_FOOTPRINT_DIR",
    ] {
        if let Ok(val) = std::env::var(var) {
            let p = PathBuf::from(val);
            if p.is_dir() && !roots.contains(&p) {
                roots.push(p);
            }
        }
    }

    roots
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

fn scan_library_dir(
    root: &Path,
    subdir: &str,
    extension: &str,
    kind: LibraryKind,
    catalog: &mut LibraryCatalog,
) {
    let dir = root.join(subdir);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let matches = match kind {
            LibraryKind::Footprint => path.is_dir() && has_extension(&path, extension),
            _ => path.is_file() && has_extension(&path, extension),
        };
        if !matches {
            continue;
        }

        if kind == LibraryKind::Symbol {
            // Register the library file without parsing — symbols are expanded lazily on search
            let lib_name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            catalog.add_source(LibrarySource::new(&lib_name, &path, kind));
        } else {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            catalog.add_source(LibrarySource::new(name, path, kind));
        }
    }
}

fn scan_3d_dir(root: &Path, catalog: &mut LibraryCatalog) {
    let dir = root.join("3dmodels");
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && has_extension(&path, "3dshapes") {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            catalog.add_source(LibrarySource::new(
                name,
                path,
                LibraryKind::ThreeDimensionalModel,
            ));
        }
    }
}

fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

#[cfg(test)]
mod tests {
    use super::{LibraryCatalog, LibraryKind, LibrarySource, detect_system_libraries};

    #[test]
    fn search_is_case_insensitive() {
        let mut catalog = LibraryCatalog::default();
        catalog.add_source(LibrarySource::new(
            "Analog Symbols",
            "fixtures/kicad/library/analog.kicad_sym",
            LibraryKind::Symbol,
        ));
        catalog.add_source(LibrarySource::new(
            "MCU Footprints",
            "fixtures/kicad/library/mcu.pretty",
            LibraryKind::Footprint,
        ));

        let results = catalog.search("analog");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name(), "Analog Symbols");
        assert_eq!(results[0].kind(), LibraryKind::Symbol);
    }

    #[test]
    fn empty_search_returns_every_source() {
        let mut catalog = LibraryCatalog::default();
        catalog.add_source(LibrarySource::new(
            "Starter symbols",
            "fixtures/kicad/library/basic.kicad_sym",
            LibraryKind::Symbol,
        ));

        assert_eq!(catalog.search("").len(), 1);
    }

    #[test]
    fn duplicate_paths_are_ignored() {
        let mut catalog = LibraryCatalog::default();
        catalog.add_source(LibrarySource::new(
            "A",
            "/tmp/a.kicad_sym",
            LibraryKind::Symbol,
        ));
        catalog.add_source(LibrarySource::new(
            "B",
            "/tmp/a.kicad_sym",
            LibraryKind::Symbol,
        ));
        assert_eq!(catalog.sources().len(), 1);
    }

    #[test]
    fn detect_system_libraries_returns_catalog() {
        // Runs without panicking on any system; may find 0 or many libraries
        let catalog = detect_system_libraries();
        // If KiCad is installed, we should find libraries
        let _ = catalog.sources().len();
    }

    #[test]
    fn by_kind_filters_correctly() {
        let mut catalog = LibraryCatalog::default();
        catalog.add_source(LibrarySource::new(
            "Sym",
            "/a.kicad_sym",
            LibraryKind::Symbol,
        ));
        catalog.add_source(LibrarySource::new(
            "Fp",
            "/b.pretty",
            LibraryKind::Footprint,
        ));
        assert_eq!(catalog.by_kind(LibraryKind::Symbol).len(), 1);
        assert_eq!(catalog.by_kind(LibraryKind::Footprint).len(), 1);
    }
}
