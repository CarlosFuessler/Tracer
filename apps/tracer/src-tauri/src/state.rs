use std::collections::HashMap;
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
    catalog.add_source(LibrarySource::new(
        "Starter symbols",
        "fixtures/kicad/library/basic.kicad_sym",
        LibraryKind::Symbol,
    ));
    catalog
}
