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

#[tauri::command]
pub fn search_symbols(query: String, state: State<'_, AppState>) -> Vec<LibraryEntry> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    let results = inner.libraries.search(&query);
    results
        .iter()
        .filter(|s| s.kind() == LibraryKind::Symbol)
        .take(100)
        .map(|s| LibraryEntry {
            name: s.name().to_owned(),
            lib_path: s.path().display().to_string(),
            symbol_name: s.symbol_name().to_owned(),
            kind: "symbol".to_owned(),
        })
        .collect()
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
