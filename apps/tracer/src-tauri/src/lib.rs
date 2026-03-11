mod commands;
mod state;

pub use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new().expect("failed to initialise editor state");

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::library::search_symbols,
            commands::library::list_symbol_libraries,
            commands::library::list_symbols_in_library,
            commands::library::get_symbol_graphics,
            commands::document::get_document,
            commands::document::place_symbol,
            commands::document::place_wire,
            commands::document::undo,
            commands::document::redo,
            commands::document::delete_objects,
            commands::document::select_objects,
            commands::document::clear_selection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
