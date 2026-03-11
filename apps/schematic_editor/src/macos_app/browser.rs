//! Component browser — right panel with search, library browsing,
//! draggable rows, and placed-objects list.

use super::SchematicEditorWindow;
use crate::ui::{drag::{DragPreview, LibraryDragItem}, theme, widgets};
use gpui::{
    AnyElement, Context, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    div, prelude::*, px,
};
use library_index::LibraryKind;

pub(super) fn component_browser(
    state: &mut SchematicEditorWindow,
    cx: &mut Context<'_, SchematicEditorWindow>,
) -> impl IntoElement {
    let query = state.search_query.clone();

    let search_results = state.bootstrap.libraries.search(&query);
    let symbol_results: Vec<AnyElement> = search_results
        .iter()
        .filter(|s| s.kind() == LibraryKind::Symbol)
        .take(30)
        .enumerate()
        .map(|(i, s)| {
            draggable_library_row(
                i,
                s.name().to_owned(),
                s.path().display().to_string(),
                s.symbol_name().to_owned(),
                "◫",
                LibraryKind::Symbol,
            )
        })
        .collect();

    let footprint_results: Vec<AnyElement> = search_results
        .iter()
        .filter(|s| s.kind() == LibraryKind::Footprint)
        .take(20)
        .enumerate()
        .map(|(i, s)| {
            draggable_library_row(
                i + 1000,
                s.name().to_owned(),
                s.path().display().to_string(),
                String::new(),
                "⬡",
                LibraryKind::Footprint,
            )
        })
        .collect();

    let object_list: Vec<AnyElement> = state
        .bootstrap
        .document
        .objects()
        .iter()
        .take(30)
        .map(|obj| {
            let selected = state.bootstrap.document.selection().contains(obj.id());
            let prefix = if selected { "● " } else { "  " };
            widgets::library_row(
                format!("{}{}", prefix, obj.display_name()),
                format!("{} at {}", obj.kind().label(), obj.position()),
            )
        })
        .collect();

    let total_libs = state.bootstrap.libraries.sources().len();
    let sym_count = symbol_results.len();
    let fp_count = footprint_results.len();

    div()
        .id("component-browser")
        .w(px(280.0))
        .flex()
        .flex_col()
        .bg(theme::BG_SURFACE)
        .border_l_1()
        .border_color(theme::BORDER_SUBTLE)
        .overflow_hidden()
        // Header
        .child(
            div()
                .px(px(12.0))
                .pt(px(10.0))
                .pb(px(6.0))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_color(theme::TEXT_PRIMARY)
                        .text_size(px(13.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child("Components"),
                )
                .child(
                    div()
                        .text_color(theme::TEXT_MUTED)
                        .text_size(px(10.0))
                        .child(format!("{total_libs} libs")),
                ),
        )
        // Search field
        .child(search_field(state, cx))
        // Results
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .p(px(8.0))
                .overflow_hidden()
                .child(widgets::panel_card(
                    "SYMBOLS",
                    symbol_results,
                    if query.is_empty() {
                        "All symbol libraries loaded."
                    } else {
                        "No matching symbols."
                    },
                ))
                .children(if fp_count > 0 || !query.is_empty() {
                    Some(widgets::panel_card(
                        "FOOTPRINTS",
                        footprint_results,
                        "No matching footprints.",
                    ))
                } else {
                    None
                })
                .child(widgets::panel_card(
                    "PLACED OBJECTS",
                    object_list,
                    "Drag components onto the canvas.",
                )),
        )
        // Summary
        .child(
            div()
                .px(px(12.0))
                .py(px(6.0))
                .border_t_1()
                .border_color(theme::BORDER_SUBTLE)
                .text_color(theme::TEXT_MUTED)
                .text_size(px(10.0))
                .child(format!(
                    "{sym_count} symbols · {fp_count} footprints{}",
                    if query.is_empty() { "" } else { " (filtered)" }
                )),
        )
}

fn search_field(
    state: &SchematicEditorWindow,
    cx: &mut Context<'_, SchematicEditorWindow>,
) -> impl IntoElement {
    let query = state.search_query.clone();
    let has_query = !query.is_empty();

    div()
        .px(px(8.0))
        .pb(px(4.0))
        .child(
            div()
                .id("search-box")
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(10.0))
                .py(px(7.0))
                .rounded(px(8.0))
                .bg(theme::BG_BASE)
                .border_1()
                .border_color(if has_query {
                    theme::ACCENT
                } else {
                    theme::BORDER_SUBTLE
                })
                .child(
                    div()
                        .text_color(theme::TEXT_MUTED)
                        .text_size(px(12.0))
                        .child("🔍"),
                )
                .child(
                    div()
                        .flex_1()
                        .text_color(if has_query {
                            theme::TEXT_PRIMARY
                        } else {
                            theme::TEXT_MUTED
                        })
                        .text_size(px(12.0))
                        .child(if has_query {
                            query
                        } else {
                            "Search components…".to_string()
                        }),
                )
                .children(if has_query {
                    Some(
                        div()
                            .id("clear-search")
                            .text_color(theme::TEXT_MUTED)
                            .text_size(px(11.0))
                            .cursor_pointer()
                            .hover(|s| s.text_color(theme::TEXT_PRIMARY))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.search_query.clear();
                                cx.notify();
                            }))
                            .child("✕"),
                    )
                } else {
                    None
                }),
        )
}

fn draggable_library_row(
    index: usize,
    name: String,
    detail: String,
    symbol_name: String,
    icon: &'static str,
    kind: LibraryKind,
) -> AnyElement {
    let drag_name = name.clone();
    let drag_lib_path = detail.clone();
    let drag_symbol_name = symbol_name;

    div()
        .id(("lib-row", index))
        .px(px(10.0))
        .py(px(6.0))
        .rounded(px(6.0))
        .cursor(gpui::CursorStyle::PointingHand)
        .hover(|s| s.bg(theme::BG_HOVER))
        .on_drag(
            LibraryDragItem {
                name: drag_name.clone(),
                kind,
                lib_path: drag_lib_path.clone(),
                symbol_name: drag_symbol_name.clone(),
            },
            move |_item, _offset, _window, cx| {
                cx.new(|_| DragPreview {
                    name: drag_name.clone(),
                    icon,
                })
            },
        )
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .w(px(28.0))
                .h(px(28.0))
                .rounded(px(6.0))
                .bg(theme::BG_BASE)
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_color(theme::ACCENT)
                        .text_size(px(14.0))
                        .child(icon),
                ),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .overflow_hidden()
                .child(
                    div()
                        .text_color(theme::TEXT_PRIMARY)
                        .text_size(px(12.0))
                        .child(name),
                )
                .child(
                    div()
                        .text_color(theme::TEXT_MUTED)
                        .text_size(px(9.0))
                        .child(detail),
                ),
        )
        .child(
            div()
                .text_color(theme::TEXT_MUTED)
                .text_size(px(10.0))
                .child("⠿"),
        )
        .into_any_element()
}
