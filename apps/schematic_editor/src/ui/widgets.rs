use gpui::{
    AnyElement, App, ClickEvent, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    Window, div, prelude::*, px,
};

use super::theme;

/// A rounded pill button used in the top bar and toolbar.
#[allow(dead_code)]
pub fn pill_button(
    id: &'static str,
    label: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let bg = if active {
        theme::ACCENT
    } else {
        theme::BG_ELEVATED
    };
    let border = if active {
        theme::ACCENT
    } else {
        theme::BORDER_SUBTLE
    };
    let text = if active {
        theme::TEXT_PRIMARY
    } else {
        theme::TEXT_SECONDARY
    };

    div()
        .id(id)
        .px(px(14.0))
        .py(px(6.0))
        .rounded(px(16.0))
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_color(text)
        .text_sm()
        .cursor_pointer()
        .hover(|s| {
            s.bg(if active {
                theme::ACCENT_HOVER
            } else {
                theme::BG_HOVER
            })
        })
        .child(label)
        .on_click(on_click)
}

/// A square tool button for the vertical tool rail — with click handler.
pub fn tool_button_interactive(
    label: &'static str,
    icon_char: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let bg = if active {
        theme::ACCENT_MUTED
    } else {
        gpui::Hsla::transparent_black()
    };
    let text = if active {
        theme::ACCENT
    } else {
        theme::TEXT_SECONDARY
    };
    let border = if active {
        theme::ACCENT
    } else {
        gpui::Hsla::transparent_black()
    };

    div()
        .id(label)
        .w(px(52.0))
        .h(px(52.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(border)
        .bg(bg)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(2.0))
        .cursor_pointer()
        .hover(|s| s.bg(theme::BG_HOVER))
        .on_click(on_click)
        .child(div().text_color(text).text_size(px(18.0)).child(icon_char))
        .child(
            div()
                .text_color(theme::TEXT_MUTED)
                .text_size(px(9.0))
                .child(label),
        )
        .into_any_element()
}

/// A square tool button for the vertical tool rail (non-interactive).
#[allow(dead_code)]
pub fn tool_button(label: &'static str, icon_char: &'static str, active: bool) -> AnyElement {
    let bg = if active {
        theme::ACCENT_MUTED
    } else {
        gpui::Hsla::transparent_black()
    };
    let text = if active {
        theme::ACCENT
    } else {
        theme::TEXT_SECONDARY
    };
    let border = if active {
        theme::ACCENT
    } else {
        gpui::Hsla::transparent_black()
    };

    div()
        .w(px(52.0))
        .h(px(52.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(border)
        .bg(bg)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(2.0))
        .cursor_pointer()
        .hover(|s| s.bg(theme::BG_HOVER))
        .child(div().text_color(text).text_size(px(18.0)).child(icon_char))
        .child(
            div()
                .text_color(theme::TEXT_MUTED)
                .text_size(px(9.0))
                .child(label),
        )
        .into_any_element()
}

/// A section header label used in side panels.
pub fn section_header(label: &'static str) -> AnyElement {
    div()
        .text_color(theme::TEXT_MUTED)
        .text_size(px(11.0))
        .pb(px(4.0))
        .child(label)
        .into_any_element()
}

/// A list row showing a library name and detail line.
pub fn library_row(name: String, detail: String) -> AnyElement {
    div()
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(6.0))
        .hover(|s| s.bg(theme::BG_HOVER))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_color(theme::TEXT_PRIMARY)
                .text_size(px(12.0))
                .child(name),
        )
        .child(
            div()
                .text_color(theme::TEXT_MUTED)
                .text_size(px(10.0))
                .child(detail),
        )
        .into_any_element()
}

/// A small status chip for the bottom bar.
pub fn status_chip(text: String) -> AnyElement {
    div()
        .px(px(8.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(theme::BG_ELEVATED)
        .text_color(theme::TEXT_SECONDARY)
        .text_size(px(11.0))
        .child(text)
        .into_any_element()
}

/// A card container with a title and body content.
pub fn panel_card(
    title: &'static str,
    children: Vec<AnyElement>,
    empty_msg: &'static str,
) -> AnyElement {
    div()
        .p(px(12.0))
        .rounded(px(8.0))
        .bg(theme::BG_SURFACE)
        .border_1()
        .border_color(theme::BORDER_SUBTLE)
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(section_header(title))
        .children(if children.is_empty() {
            vec![
                div()
                    .text_color(theme::TEXT_MUTED)
                    .text_size(px(11.0))
                    .child(empty_msg)
                    .into_any_element(),
            ]
        } else {
            children
        })
        .into_any_element()
}
