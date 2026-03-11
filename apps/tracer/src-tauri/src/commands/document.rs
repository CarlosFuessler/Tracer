use serde::Serialize;
use tauri::State;

use eda_core::{
    EditorCommand, EntityId, Point2D, SchematicObject, SchematicObjectKind, SymbolGraphics,
    WireSegment,
};

use crate::AppState;

// ── DTOs sent to the frontend ─────────────────────────────────

#[derive(Serialize)]
pub struct ObjectDto {
    pub id: u64,
    pub kind: SchematicObjectKind,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub wire: Option<WireDto>,
    pub graphics: Option<SymbolGraphics>,
    pub selected: bool,
}

#[derive(Serialize)]
pub struct WireDto {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

#[derive(Serialize)]
pub struct DocumentDto {
    pub objects: Vec<ObjectDto>,
    pub can_undo: bool,
    pub can_redo: bool,
}

// ── Helpers ───────────────────────────────────────────────────

fn to_dto(obj: &eda_core::SchematicObject, selected: bool) -> ObjectDto {
    let wire = obj.wire_segment().map(|w| WireDto {
        x1: w.start.x,
        y1: w.start.y,
        x2: w.end.x,
        y2: w.end.y,
    });
    ObjectDto {
        id: obj.id().raw(),
        kind: obj.kind(),
        name: obj.display_name().to_owned(),
        x: obj.position().x,
        y: obj.position().y,
        wire,
        graphics: obj.symbol_graphics().cloned(),
        selected,
    }
}

fn build_doc_dto(state: &crate::state::EditorState) -> DocumentDto {
    let objects = state
        .document
        .objects()
        .iter()
        .map(|o| {
            let sel = state.document.selection().contains(o.id());
            to_dto(o, sel)
        })
        .collect();
    DocumentDto {
        objects,
        can_undo: state.commands.can_undo(),
        can_redo: state.commands.can_redo(),
    }
}

// ── Commands ──────────────────────────────────────────────────

#[tauri::command]
pub fn get_document(state: State<'_, AppState>) -> DocumentDto {
    let inner = state.inner.lock().expect("state lock poisoned");
    build_doc_dto(&inner)
}

#[tauri::command]
pub fn place_symbol(
    lib_path: String,
    symbol_name: String,
    x: f64,
    y: f64,
    state: State<'_, AppState>,
) -> Result<DocumentDto, String> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    let graphics = inner.load_symbol_graphics(&lib_path, &symbol_name);

    let id = inner.ids.next_entity_id();
    let display = if symbol_name.is_empty() {
        lib_path
            .split('/')
            .next_back()
            .unwrap_or("symbol")
            .to_owned()
    } else {
        symbol_name.clone()
    };

    let mut obj = SchematicObject::symbol(id, display, Point2D::new(x, y));
    if let Some(g) = graphics {
        obj.set_symbol_graphics(g);
    }

    inner
        .apply(EditorCommand::PlaceObject { object: obj })
        .map_err(|e| e.to_string())?;

    Ok(build_doc_dto(&inner))
}

#[tauri::command]
pub fn place_wire(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    state: State<'_, AppState>,
) -> Result<DocumentDto, String> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    let id = inner.ids.next_entity_id();
    let seg = WireSegment::new(Point2D::new(x1, y1), Point2D::new(x2, y2));
    let obj = SchematicObject::wire(id, seg);
    inner
        .apply(EditorCommand::PlaceObject { object: obj })
        .map_err(|e| e.to_string())?;
    Ok(build_doc_dto(&inner))
}

#[tauri::command]
pub fn undo(state: State<'_, AppState>) -> Result<DocumentDto, String> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    inner.undo().map_err(|e| e.to_string())?;
    Ok(build_doc_dto(&inner))
}

#[tauri::command]
pub fn redo(state: State<'_, AppState>) -> Result<DocumentDto, String> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    inner.redo().map_err(|e| e.to_string())?;
    Ok(build_doc_dto(&inner))
}

#[tauri::command]
pub fn delete_objects(
    ids: Vec<u64>,
    state: State<'_, AppState>,
) -> Result<DocumentDto, String> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    for raw_id in ids {
        let id = EntityId::new(raw_id);
        inner
            .apply(EditorCommand::DeleteObject { id })
            .map_err(|e| e.to_string())?;
    }
    Ok(build_doc_dto(&inner))
}

#[tauri::command]
pub fn select_objects(
    ids: Vec<u64>,
    state: State<'_, AppState>,
) -> Result<DocumentDto, String> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    let entity_ids: Vec<EntityId> = ids.into_iter().map(EntityId::new).collect();
    inner
        .apply(EditorCommand::ReplaceSelection { ids: entity_ids })
        .map_err(|e| e.to_string())?;
    Ok(build_doc_dto(&inner))
}

#[tauri::command]
pub fn clear_selection(state: State<'_, AppState>) -> Result<DocumentDto, String> {
    let mut inner = state.inner.lock().expect("state lock poisoned");
    inner
        .apply(EditorCommand::ClearSelection)
        .map_err(|e| e.to_string())?;
    Ok(build_doc_dto(&inner))
}

