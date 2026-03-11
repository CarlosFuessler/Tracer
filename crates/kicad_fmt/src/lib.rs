#![forbid(unsafe_code)]

pub mod symbol_parser;

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureKind {
    Project,
    Schematic,
    SymbolLibrary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixtureInfo {
    pub name: &'static str,
    pub path: &'static str,
    pub kind: FixtureKind,
}

pub const PROJECT_FIXTURE: &str = include_str!("../../../fixtures/kicad/project/basic.kicad_pro");
pub const SCHEMATIC_FIXTURE: &str =
    include_str!("../../../fixtures/kicad/schematic/basic.kicad_sch");
pub const SYMBOL_LIBRARY_FIXTURE: &str =
    include_str!("../../../fixtures/kicad/library/basic.kicad_sym");

const FIXTURE_CATALOG: [FixtureInfo; 3] = [
    FixtureInfo {
        name: "basic.kicad_pro",
        path: "fixtures/kicad/project/basic.kicad_pro",
        kind: FixtureKind::Project,
    },
    FixtureInfo {
        name: "basic.kicad_sch",
        path: "fixtures/kicad/schematic/basic.kicad_sch",
        kind: FixtureKind::Schematic,
    },
    FixtureInfo {
        name: "basic.kicad_sym",
        path: "fixtures/kicad/library/basic.kicad_sym",
        kind: FixtureKind::SymbolLibrary,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KicadDocumentKind {
    Project,
    Schematic,
    SymbolLibrary,
}

impl KicadDocumentKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Schematic => "schematic",
            Self::SymbolLibrary => "symbol library",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KicadDocumentSummary {
    path: PathBuf,
    kind: KicadDocumentKind,
    symbol_count: usize,
    wire_count: usize,
    label_count: usize,
    junction_count: usize,
    sheet_count: usize,
}

impl KicadDocumentSummary {
    #[must_use]
    fn new(path: PathBuf, kind: KicadDocumentKind) -> Self {
        Self {
            path,
            kind,
            symbol_count: 0,
            wire_count: 0,
            label_count: 0,
            junction_count: 0,
            sheet_count: 0,
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub const fn kind(&self) -> KicadDocumentKind {
        self.kind
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
    }

    #[must_use]
    pub const fn symbol_count(&self) -> usize {
        self.symbol_count
    }

    #[must_use]
    pub const fn wire_count(&self) -> usize {
        self.wire_count
    }

    #[must_use]
    pub const fn label_count(&self) -> usize {
        self.label_count
    }

    #[must_use]
    pub const fn junction_count(&self) -> usize {
        self.junction_count
    }

    #[must_use]
    pub const fn sheet_count(&self) -> usize {
        self.sheet_count
    }

    #[must_use]
    pub fn detail_summary(&self) -> String {
        match self.kind {
            KicadDocumentKind::Project => {
                if self.sheet_count == 0 {
                    "project metadata".to_string()
                } else {
                    format!("{} schematic sheet(s)", self.sheet_count)
                }
            }
            KicadDocumentKind::Schematic => format!(
                "{} symbol(s), {} wire(s), {} label(s), {} junction(s)",
                self.symbol_count, self.wire_count, self.label_count, self.junction_count
            ),
            KicadDocumentKind::SymbolLibrary => {
                format!("{} symbol definition(s)", self.symbol_count)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportCatalog {
    roots: Vec<PathBuf>,
    documents: Vec<KicadDocumentSummary>,
}

impl ImportCatalog {
    #[must_use]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    #[must_use]
    pub fn documents(&self) -> &[KicadDocumentSummary] {
        &self.documents
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    #[must_use]
    pub fn total_by_kind(&self, kind: KicadDocumentKind) -> usize {
        self.documents
            .iter()
            .filter(|document| document.kind() == kind)
            .count()
    }
}

#[derive(Debug)]
pub enum ImportError {
    ReadDirectory { path: PathBuf, source: io::Error },
    ReadFile { path: PathBuf, source: io::Error },
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDirectory { path, source } => {
                write!(f, "failed to read directory {}: {source}", path.display())
            }
            Self::ReadFile { path, source } => {
                write!(f, "failed to read file {}: {source}", path.display())
            }
        }
    }
}

impl Error for ImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. } | Self::ReadFile { source, .. } => Some(source),
        }
    }
}

#[must_use]
pub const fn fixture_catalog() -> &'static [FixtureInfo] {
    &FIXTURE_CATALOG
}

#[must_use]
pub fn looks_like_kicad_document(input: &str) -> bool {
    let trimmed = input.trim_start();

    trimmed.starts_with("(kicad_") || (trimmed.starts_with('{') && trimmed.contains("\"version\""))
}

pub fn import_from_paths(paths: &[PathBuf]) -> Result<ImportCatalog, ImportError> {
    let mut documents = Vec::new();

    for path in paths {
        collect_kicad_documents(path, &mut documents)?;
    }

    documents.sort_by(|left, right| left.path().cmp(right.path()));

    Ok(ImportCatalog {
        roots: paths.to_vec(),
        documents,
    })
}

fn collect_kicad_documents(
    path: &Path,
    documents: &mut Vec<KicadDocumentSummary>,
) -> Result<(), ImportError> {
    if path.is_dir() {
        let directory_entries =
            fs::read_dir(path).map_err(|source| ImportError::ReadDirectory {
                path: path.to_path_buf(),
                source,
            })?;

        let mut child_paths = Vec::new();
        for entry in directory_entries {
            let entry = entry.map_err(|source| ImportError::ReadDirectory {
                path: path.to_path_buf(),
                source,
            })?;
            child_paths.push(entry.path());
        }

        child_paths.sort();

        for child_path in child_paths {
            collect_kicad_documents(&child_path, documents)?;
        }

        return Ok(());
    }

    let Some(kind) = document_kind_from_path(path) else {
        return Ok(());
    };

    let contents = fs::read_to_string(path).map_err(|source| ImportError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;

    documents.push(summarize_document(path.to_path_buf(), kind, &contents));
    Ok(())
}

fn document_kind_from_path(path: &Path) -> Option<KicadDocumentKind> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("kicad_pro") => Some(KicadDocumentKind::Project),
        Some("kicad_sch") => Some(KicadDocumentKind::Schematic),
        Some("kicad_sym") => Some(KicadDocumentKind::SymbolLibrary),
        _ => None,
    }
}

fn summarize_document(
    path: PathBuf,
    kind: KicadDocumentKind,
    contents: &str,
) -> KicadDocumentSummary {
    let mut summary = KicadDocumentSummary::new(path, kind);

    match kind {
        KicadDocumentKind::Project => {
            summary.sheet_count = contents.matches("\n    [").count();
        }
        KicadDocumentKind::Schematic => {
            let counts = summarize_schematic(contents);
            summary.symbol_count = counts.symbol_count;
            summary.wire_count = counts.wire_count;
            summary.label_count = counts.label_count;
            summary.junction_count = counts.junction_count;
        }
        KicadDocumentKind::SymbolLibrary => {
            summary.symbol_count = summarize_symbol_library(contents);
        }
    }

    summary
}

#[derive(Default)]
struct SchematicCounts {
    symbol_count: usize,
    wire_count: usize,
    label_count: usize,
    junction_count: usize,
}

fn summarize_schematic(contents: &str) -> SchematicCounts {
    let mut counts = SchematicCounts::default();
    let mut depth = 0i32;
    let mut lib_symbols_depth = None;

    for line in contents.lines() {
        let trimmed = line.trim_start();
        let inside_lib_symbols = lib_symbols_depth.is_some();

        if !inside_lib_symbols {
            if trimmed == "(symbol" || trimmed.starts_with("(symbol ") {
                counts.symbol_count += 1;
            } else if trimmed.starts_with("(wire") {
                counts.wire_count += 1;
            } else if trimmed.starts_with("(label ")
                || trimmed.starts_with("(global_label")
                || trimmed.starts_with("(hierarchical_label")
            {
                counts.label_count += 1;
            } else if trimmed.starts_with("(junction") {
                counts.junction_count += 1;
            }
        }

        let opened = line.chars().filter(|character| *character == '(').count() as i32;
        let closed = line.chars().filter(|character| *character == ')').count() as i32;
        let next_depth = depth + opened - closed;

        if trimmed.starts_with("(lib_symbols") && next_depth > depth {
            lib_symbols_depth = Some(next_depth);
        }

        depth = next_depth;

        if let Some(marker) = lib_symbols_depth
            && depth < marker
        {
            lib_symbols_depth = None;
        }
    }

    counts
}

fn summarize_symbol_library(contents: &str) -> usize {
    let mut definition_count = 0;
    let mut depth = 0i32;

    for line in contents.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("(symbol \"") && depth == 1 {
            definition_count += 1;
        }

        let opened = line.chars().filter(|character| *character == '(').count() as i32;
        let closed = line.chars().filter(|character| *character == ')').count() as i32;
        depth += opened - closed;
    }

    definition_count
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        KicadDocumentKind, PROJECT_FIXTURE, SCHEMATIC_FIXTURE, SYMBOL_LIBRARY_FIXTURE,
        fixture_catalog, import_from_paths, looks_like_kicad_document,
    };

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/kicad")
    }

    #[test]
    fn fixture_catalog_covers_bootstrap_files() {
        assert_eq!(fixture_catalog().len(), 3);
        assert!(
            fixture_catalog()
                .iter()
                .any(|fixture| fixture.name == "basic.kicad_sch")
        );
    }

    #[test]
    fn bundled_fixtures_look_like_kicad_documents() {
        assert!(looks_like_kicad_document(PROJECT_FIXTURE));
        assert!(looks_like_kicad_document(SCHEMATIC_FIXTURE));
        assert!(looks_like_kicad_document(SYMBOL_LIBRARY_FIXTURE));
    }

    #[test]
    fn import_catalog_scans_fixture_directory_recursively() {
        let catalog = match import_from_paths(&[fixture_root()]) {
            Ok(catalog) => catalog,
            Err(error) => panic!("fixture directory should import: {error}"),
        };

        assert_eq!(catalog.documents().len(), 3);
        assert_eq!(catalog.total_by_kind(KicadDocumentKind::Project), 1);
        assert_eq!(catalog.total_by_kind(KicadDocumentKind::Schematic), 1);
        assert_eq!(catalog.total_by_kind(KicadDocumentKind::SymbolLibrary), 1);
    }

    #[test]
    fn schematic_fixture_summary_counts_visible_items() {
        let fixture_path = fixture_root().join("schematic/basic.kicad_sch");
        let catalog = match import_from_paths(&[fixture_path]) {
            Ok(catalog) => catalog,
            Err(error) => panic!("fixture schematic should import: {error}"),
        };
        let summary = &catalog.documents()[0];

        assert_eq!(summary.kind(), KicadDocumentKind::Schematic);
        assert_eq!(summary.symbol_count(), 1);
        assert_eq!(summary.wire_count(), 1);
    }

    #[test]
    fn symbol_library_fixture_counts_top_level_symbols() {
        let fixture_path = fixture_root().join("library/basic.kicad_sym");
        let catalog = match import_from_paths(&[fixture_path]) {
            Ok(catalog) => catalog,
            Err(error) => panic!("fixture symbol library should import: {error}"),
        };
        let summary = &catalog.documents()[0];

        assert_eq!(summary.kind(), KicadDocumentKind::SymbolLibrary);
        assert_eq!(summary.symbol_count(), 1);
    }
}
