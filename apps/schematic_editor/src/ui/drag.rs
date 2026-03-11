//! Drag-and-drop types for the schematic editor.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use library_index::LibraryKind;

use super::theme;

/// Payload carried during a library drag operation.
#[derive(Debug, Clone)]
pub struct LibraryDragItem {
    pub name: String,
    #[allow(dead_code)]
    pub kind: LibraryKind,
    /// Path to the .kicad_sym file containing this symbol.
    pub lib_path: String,
    /// The symbol name inside the library file (for loading graphics).
    pub symbol_name: String,
}

/// A lightweight view rendered as the drag ghost.
pub struct DragPreview {
    pub name: String,
    pub icon: &'static str,
}

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .px(px(12.0))
            .py(px(6.0))
            .rounded(px(8.0))
            .bg(theme::ACCENT_MUTED)
            .border_1()
            .border_color(theme::ACCENT)
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                div()
                    .text_color(theme::ACCENT)
                    .text_size(px(14.0))
                    .child(self.icon),
            )
            .child(
                div()
                    .text_color(theme::TEXT_PRIMARY)
                    .text_size(px(12.0))
                    .child(self.name.clone()),
            )
    }
}
