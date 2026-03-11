use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use eda_core::{CommandStack, IdGenerator, ProjectDocument, ProjectManifest, APP_METADATA};
use library_index::{LibraryCatalog, LibraryKind, LibrarySource, detect_system_libraries};

pub struct AppState {
    pub inner: Mutex<EditorState>,
}

pub struct EditorState {
    pub document: ProjectDocument,
    pub ids: IdGenerator,
    pub commands: CommandStack,
    pub libraries: LibraryCatalog,
    pub symbol_cache: HashMap<(PathBuf, String), eda_core::SymbolGraphics>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut ids = IdGenerator::default();
        let manifest = ProjectManifest::new(APP_METADATA.product_name);
        let document = ProjectDocument::new(ids.next_project_id(), manifest);
        Ok(Self {
            inner: Mutex::new(EditorState {
                document,
                ids,
                commands: CommandStack::default(),
                libraries: build_library_catalog(),
                symbol_cache: HashMap::new(),
            }),
        })
    }
}

impl EditorState {
    pub fn load_symbol_graphics(
        &mut self,
        lib_path: &str,
        symbol_name: &str,
    ) -> Option<eda_core::SymbolGraphics> {
        if lib_path.is_empty() || symbol_name.is_empty() {
            return None;
        }
        let key = (PathBuf::from(lib_path), symbol_name.to_owned());
        if let Some(cached) = self.symbol_cache.get(&key) {
            return Some(cached.clone());
        }
        let graphics = kicad_fmt::symbol_parser::parse_one_symbol(&key.0, &key.1)?;
        self.symbol_cache.insert(key, graphics.clone());
        Some(graphics)
    }

    /// Apply a command — uses destructuring to satisfy the borrow checker.
    pub fn apply(&mut self, cmd: eda_core::EditorCommand) -> Result<(), eda_core::DocumentError> {
        let Self { ref mut commands, ref mut document, .. } = *self;
        commands.apply(document, cmd)
    }

    /// Undo the last command.
    pub fn undo(&mut self) -> Result<bool, eda_core::DocumentError> {
        let Self { ref mut commands, ref mut document, .. } = *self;
        commands.undo(document)
    }

    /// Redo the last undone command.
    pub fn redo(&mut self) -> Result<bool, eda_core::DocumentError> {
        let Self { ref mut commands, ref mut document, .. } = *self;
        commands.redo(document)
    }
}

fn build_library_catalog() -> LibraryCatalog {
    let mut catalog = detect_system_libraries();
    if let Some(starter_path) = starter_symbol_library_path() {
        catalog.add_source(LibrarySource::new(
            "Starter symbols",
            starter_path,
            LibraryKind::Symbol,
        ));
    } else {
        eprintln!("warning: starter symbol library is unavailable");
    }
    catalog
}

fn starter_symbol_library_path() -> Option<PathBuf> {
    let repo_fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/kicad/library/basic.kicad_sym");
    if repo_fixture.is_file() {
        return match repo_fixture.canonicalize() {
            Ok(path) => Some(path),
            Err(error) => {
                eprintln!(
                    "warning: failed to canonicalize starter library {}: {error}",
                    repo_fixture.display()
                );
                Some(repo_fixture)
            }
        };
    }

    let fixture_dir = std::env::temp_dir().join("rust_pcb_editor");
    if let Err(error) = fs::create_dir_all(&fixture_dir) {
        eprintln!(
            "warning: failed to prepare starter library directory {}: {error}",
            fixture_dir.display()
        );
        return None;
    }

    let materialized_fixture = fixture_dir.join("basic.kicad_sym");
    if let Err(error) = fs::write(&materialized_fixture, kicad_fmt::SYMBOL_LIBRARY_FIXTURE) {
        eprintln!(
            "warning: failed to write starter library {}: {error}",
            materialized_fixture.display()
        );
        return None;
    }

    Some(materialized_fixture)
}

#[cfg(test)]
mod tests {
    use library_index::LibraryKind;

    use super::{build_library_catalog, starter_symbol_library_path};

    #[test]
    fn starter_library_fixture_is_available() {
        let path = starter_symbol_library_path().expect("starter library should resolve");
        assert!(path.is_file(), "starter library path should exist");
        assert!(
            !kicad_fmt::symbol_parser::list_symbol_names(&path).is_empty(),
            "starter library should contain at least one symbol"
        );
    }

    #[test]
    fn tracer_catalog_includes_starter_symbols() {
        let catalog = build_library_catalog();
        assert!(catalog.sources().iter().any(|source| {
            source.kind() == LibraryKind::Symbol && source.name() == "Starter symbols"
        }));
    }
}
