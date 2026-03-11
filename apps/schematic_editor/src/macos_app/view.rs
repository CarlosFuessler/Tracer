use super::SchematicEditorWindow;
use super::{browser, canvas_view};
use crate::canvas::CanvasTool;
use crate::ui::{theme, widgets};
use gpui::{
    Context, IntoElement, ParentElement, Styled, div, prelude::*, px,
};

pub(super) fn render(
    state: &mut SchematicEditorWindow,
    cx: &mut Context<'_, SchematicEditorWindow>,
) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(theme::BG_BASE)
        .text_color(theme::TEXT_PRIMARY)
        .text_size(px(13.0))
        // Global key handler: route typing to search, shortcuts to tools
        .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = &event.keystroke.key;
            let mods = &event.keystroke.modifiers;

            // Let platform shortcuts through (Cmd+Z, Cmd+Q, etc.)
            if mods.platform || mods.control {
                return;
            }

            // Tool shortcuts only when search is empty
            if this.search_query.is_empty() {
                match key.as_str() {
                    "v" => { this.set_tool(CanvasTool::Select, cx); return; }
                    "w" => { this.set_tool(CanvasTool::Wire, cx); return; }
                    "l" => { this.set_tool(CanvasTool::Label, cx); return; }
                    "p" => { this.set_tool(CanvasTool::Place, cx); return; }
                    "m" => { this.set_tool(CanvasTool::Move, cx); return; }
                    _ => {}
                }
            }

            if key == "backspace" {
                this.search_query.pop();
                cx.notify();
            } else if key == "escape" {
                if !this.search_query.is_empty() {
                    this.search_query.clear();
                } else {
                    this.handle_escape(cx);
                }
                cx.notify();
            } else if let Some(ch) = &event.keystroke.key_char
                && ch.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' || c == '.')
            {
                this.search_query.push_str(ch);
                cx.notify();
            }
        }))
        .child(top_bar(state, cx))
        .child(
            div()
                .flex()
                .flex_1()
                .overflow_hidden()
                .child(tool_rail(state, cx))
                .child(canvas_view::canvas_area(state, cx))
                .child(browser::component_browser(state, cx)),
        )
        .child(status_bar(state))
}

// ── Top bar ────────────────────────────────────────────

fn top_bar(
    _state: &SchematicEditorWindow,
    cx: &mut Context<'_, SchematicEditorWindow>,
) -> impl IntoElement {
    div()
        .h(px(40.0))
        .flex()
        .items_center()
        .justify_between()
        .px(px(16.0))
        .bg(theme::BG_SURFACE)
        .border_b_1()
        .border_color(theme::BORDER_SUBTLE)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(
                    div()
                        .text_color(theme::ACCENT)
                        .text_size(px(14.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("⬡ Tracer"),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .id("import-btn")
                        .px(px(10.0))
                        .py(px(5.0))
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(theme::BG_HOVER))
                        .text_color(theme::TEXT_SECONDARY)
                        .text_size(px(11.0))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.prompt_for_import(window, cx);
                        }))
                        .child("📂 Import"),
                )
                .child(
                    div()
                        .id("refresh-btn")
                        .px(px(10.0))
                        .py(px(5.0))
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(theme::BG_HOVER))
                        .text_color(theme::TEXT_SECONDARY)
                        .text_size(px(11.0))
                        .on_click(cx.listener(|this, _, _window, cx| {
                            this.bootstrap.refresh_imported_libraries();
                            let count = this.bootstrap.libraries.sources().len();
                            this.status_message = format!("Refreshed — {count} libraries indexed");
                            cx.notify();
                        }))
                        .child("🔄 Refresh"),
                ),
        )
}

// ── Tool rail ──────────────────────────────────────────

fn tool_rail(
    state: &SchematicEditorWindow,
    cx: &mut Context<'_, SchematicEditorWindow>,
) -> impl IntoElement {
    let active_tool = state.canvas.tool;

    div()
        .w(px(56.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(2.0))
        .py(px(8.0))
        .bg(theme::BG_SURFACE)
        .border_r_1()
        .border_color(theme::BORDER_SUBTLE)
        .children(CanvasTool::SKETCH_TOOLS.iter().copied().map(|tool| {
            widgets::tool_button_interactive(
                tool.label(),
                tool.icon(),
                active_tool == tool,
                cx.listener(move |this, _, _window, cx| {
                    this.set_tool(tool, cx);
                }),
            )
        }))
}

// ── Status bar ─────────────────────────────────────────

fn status_bar(state: &SchematicEditorWindow) -> impl IntoElement {
    div()
        .h(px(24.0))
        .flex()
        .items_center()
        .justify_between()
        .px(px(16.0))
        .bg(theme::BG_SURFACE)
        .border_t_1()
        .border_color(theme::BORDER_SUBTLE)
        .child(
            div()
                .text_color(theme::TEXT_MUTED)
                .text_size(px(10.0))
                .child(state.status_message.clone()),
        )
        .child(div().flex().gap(px(8.0)).children(vec![
            widgets::status_chip(format!(
                "Objects: {}",
                state.bootstrap.document.objects().len()
            )),
            widgets::status_chip(format!(
                "Selected: {}",
                state.bootstrap.document.selection().len()
            )),
        ]))
}
