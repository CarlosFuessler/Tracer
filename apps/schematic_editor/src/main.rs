use std::{collections::HashMap, error::Error, path::PathBuf};

use app_shell::AppShell;
use eda_core::{
    APP_METADATA, ClipboardBuffer, CommandStack, IdGenerator, ProjectDocument,
    ProjectManifest,
};
use kicad_fmt::{ImportCatalog, KicadDocumentKind};
use library_index::{LibraryCatalog, LibraryKind, LibrarySource, detect_system_libraries};
use render_scene::SceneBootstrap;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
mod canvas;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
mod macos_app;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
mod ui;

#[derive(Debug, Clone)]
pub(crate) struct EditorBootstrap {
    shell: AppShell,
    scene: SceneBootstrap,
    document: ProjectDocument,
    ids: IdGenerator,
    commands: CommandStack,
    libraries: LibraryCatalog,
    #[allow(dead_code)]
    clipboard: ClipboardBuffer,
    imports: ImportCatalog,
    /// Cache of parsed symbol graphics keyed by (file_path, symbol_name).
    symbol_cache: HashMap<(PathBuf, String), eda_core::SymbolGraphics>,
}

impl EditorBootstrap {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut ids = IdGenerator::default();
        let manifest = ProjectManifest::new(APP_METADATA.product_name);
        let shell = AppShell::new(manifest);
        let scene = SceneBootstrap::for_workspace(shell.active_workspace());
        let document = ProjectDocument::new(ids.next_project_id(), shell.manifest().clone());
        let commands = CommandStack::default();
        let clipboard = ClipboardBuffer::default();

        Ok(Self {
            shell,
            scene,
            document,
            ids,
            commands,
            libraries: auto_detect_libraries(),
            clipboard,
            imports: ImportCatalog::default(),
            symbol_cache: HashMap::new(),
        })
    }

    fn sync_scene_to_workspace(&mut self) {
        self.scene = SceneBootstrap::for_workspace(self.shell.active_workspace());
    }

    fn refresh_imported_libraries(&mut self) {
        self.libraries = auto_detect_libraries();

        for document in self
            .imports
            .documents()
            .iter()
            .filter(|document| document.kind() == KicadDocumentKind::SymbolLibrary)
        {
            self.libraries.add_source(LibrarySource::new(
                document.display_name().to_owned(),
                PathBuf::from(document.path()),
                LibraryKind::Symbol,
            ));
        }
    }

    /// Load symbol graphics from cache, or parse from disk and cache.
    fn load_symbol_graphics(
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
}

fn auto_detect_libraries() -> LibraryCatalog {
    let mut catalog = detect_system_libraries();
    // Always include the bundled fixture library
    catalog.add_source(LibrarySource::new(
        "Starter symbols",
        "fixtures/kicad/library/basic.kicad_sym",
        LibraryKind::Symbol,
    ));
    catalog
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn main() -> Result<(), Box<dyn Error>> {
    let bootstrap = EditorBootstrap::new()?;
    macos_app::run(bootstrap);
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn main() -> Result<(), Box<dyn Error>> {
    let bootstrap = EditorBootstrap::new()?;

    println!("{}", bootstrap.shell.startup_banner());
    println!("{}", APP_METADATA.tagline);
    println!(
        "Canvas: {} | indexed libraries: {}",
        bootstrap.scene.summary(),
        bootstrap.libraries.sources().len()
    );
    println!(
        "Document: {} items | selection: {} | clipboard: {}",
        bootstrap.document.objects().len(),
        bootstrap.document.selection().len(),
        bootstrap.clipboard.objects().len()
    );
    println!(
        "Native GUI is currently implemented for desktop builds on macOS, Linux, and Windows."
    );

    Ok(())
}
