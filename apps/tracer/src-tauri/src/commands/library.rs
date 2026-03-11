use std::collections::HashSet;
use std::path::Path;

use library_index::LibraryKind;
use serde::Serialize;
use tauri::State;

use crate::AppState;

#[derive(Serialize)]
pub struct LibraryEntry {
    pub name: String,
    pub lib_path: String,
    pub symbol_name: String,
    pub kind: String,
}

#[derive(Serialize)]
pub struct LibraryGroup {
    pub name: String,
    pub lib_path: String,
}

#[tauri::command]
pub fn search_symbols(query: String, state: State<'_, AppState>) -> Vec<LibraryEntry> {
    let query = query.trim().to_owned();
    if query.is_empty() {
        return Vec::new();
    }

    let mut inner = state.inner.lock().expect("state lock poisoned");
    let mut entries: Vec<_> = inner
        .libraries
        .search_symbols(&query)
        .iter()
        .map(|s| LibraryEntry {
            name: s.symbol_name().to_owned(),
            lib_path: s.path().display().to_string(),
            symbol_name: s.symbol_name().to_owned(),
            kind: "symbol".to_owned(),
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries.truncate(150);
    entries
}

#[tauri::command]
pub fn list_symbol_libraries(state: State<'_, AppState>) -> Vec<LibraryGroup> {
    let inner = state.inner.lock().expect("state lock poisoned");
    let mut seen = HashSet::new();
    let mut libraries: Vec<_> = inner
        .libraries
        .sources()
        .iter()
        .filter(|source| source.kind() == LibraryKind::Symbol)
        .filter_map(|source| {
            let path = source.path().display().to_string();
            if !seen.insert(path.clone()) {
                return None;
            }

            let name = source
                .path()
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| source.name().to_owned());

            Some(LibraryGroup {
                name,
                lib_path: path,
            })
        })
        .collect();
    libraries.sort_by(|a, b| a.name.cmp(&b.name));
    libraries
}

#[tauri::command]
pub fn list_symbols_in_library(
    lib_path: String,
    state: State<'_, AppState>,
) -> Vec<LibraryEntry> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    let normalized = Path::new(&lib_path);

    let mut entries: Vec<_> = inner
        .libraries
        .symbols_in_library(normalized)
        .iter()
        .map(|source| LibraryEntry {
            name: source.symbol_name().to_owned(),
            lib_path: source.path().display().to_string(),
            symbol_name: source.symbol_name().to_owned(),
            kind: "symbol".to_owned(),
        })
        .collect();

    if entries.is_empty() {
        entries = kicad_fmt::symbol_parser::list_symbol_names(normalized)
            .into_iter()
            .map(|symbol_name| LibraryEntry {
                name: symbol_name.clone(),
                lib_path: lib_path.clone(),
                symbol_name,
                kind: "symbol".to_owned(),
            })
            .collect();
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

#[tauri::command]
pub fn get_symbol_graphics(
    lib_path: String,
    symbol_name: String,
    state: State<'_, AppState>,
) -> Option<eda_core::SymbolGraphics> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    inner.load_symbol_graphics(&lib_path, &symbol_name)
}
