//! Canvas rendering — grid dots, objects (symbols, wires, labels, junctions),
//! and the interactive canvas div with mouse/keyboard handlers.

use super::SchematicEditorWindow;
use crate::ui::{drag::LibraryDragItem, theme};
use eda_core::SchematicObjectKind;
use gpui::{
    AnyElement, Context, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement,
    ScrollWheelEvent, Styled, div, prelude::*, px,
};

pub(super) fn canvas_area(
    state: &mut SchematicEditorWindow,
    cx: &mut Context<'_, SchematicEditorWindow>,
) -> impl IntoElement {
    let canvas = &state.canvas;
    let zoom = canvas.zoom;
    let pan = canvas.pan;
    let viewport = canvas.viewport_px;
    let grid_mm = canvas.grid_mm;
    let wire_start = canvas.wire_start;
    let active_tool = canvas.tool;
    let mouse_sch = canvas.mouse_schematic;

    let objects: Vec<_> = state
        .bootstrap
        .document
        .objects()
        .iter()
        .map(|obj| RenderedObject {
            kind: obj.kind(),
            name: obj.display_name().to_string(),
            pos: obj.position(),
            wire_seg: obj.wire_segment().copied(),
            selected: state.bootstrap.document.selection().contains(obj.id()),
            graphics: obj.symbol_graphics().cloned(),
        })
        .collect();

    let cx_half_w = viewport.0 / 2.0;
    let cx_half_h = viewport.1 / 2.0;

    let grid_children = build_grid_dots(pan, zoom, grid_mm, cx_half_w, cx_half_h);

    let object_children: Vec<AnyElement> = objects
        .iter()
        .filter(|obj| {
            // Viewport culling: skip objects far outside the visible area
            let sx = (obj.pos.x - pan.x) * zoom + cx_half_w;
            let sy = (obj.pos.y - pan.y) * zoom + cx_half_h;
            let margin = 200.0; // generous margin for large symbols
            sx > -margin
                && sx < viewport.0 + margin
                && sy > -margin
                && sy < viewport.1 + margin
        })
        .map(|obj| render_object(obj, pan, zoom, cx_half_w, cx_half_h))
        .collect();

    let wire_preview = wire_start.map(|start| {
        render_wire_preview(start, mouse_sch, pan, zoom, cx_half_w, cx_half_h)
    });

    let tool_hint = div()
        .absolute()
        .top(px(8.0))
        .left(px(8.0))
        .px(px(8.0))
        .py(px(4.0))
        .rounded(px(6.0))
        .bg(theme::BG_SURFACE)
        .text_color(theme::TEXT_SECONDARY)
        .text_size(px(11.0))
        .child(format!(
            "{} · {:.0}% · {} objects",
            active_tool.label(),
            zoom / 8.0 * 100.0,
            objects.len()
        ));

    let coord_hint = div()
        .absolute()
        .bottom(px(8.0))
        .left(px(8.0))
        .px(px(8.0))
        .py(px(4.0))
        .rounded(px(6.0))
        .bg(theme::BG_SURFACE)
        .text_color(theme::TEXT_MUTED)
        .text_size(px(10.0))
        .child(format!(
            "X: {:.2} mm  Y: {:.2} mm",
            mouse_sch.x, mouse_sch.y
        ));

    let entity = cx.weak_entity();

    div()
        .id("schematic-canvas")
        .flex_1()
        .relative()
        .overflow_hidden()
        .bg(theme::CANVAS_BG)
        .cursor(gpui::CursorStyle::Crosshair)
        .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, _window, cx| {
            let pos = event.position;
            this.canvas_click(f32::from(pos.x) as f64, f32::from(pos.y) as f64, cx);
        }))
        .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, _cx| {
            let pos = event.position;
            this.canvas.mouse_schematic = this.canvas.to_schematic(
                f32::from(pos.x) as f64,
                f32::from(pos.y) as f64,
            );
            // Only notify during wire drawing (need preview update)
            if this.canvas.wire_start.is_some() {
                _cx.notify();
            }
        }))
        .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
            let delta = -f32::from(event.delta.pixel_delta(px(1.0)).y) as f64;
            let pos = event.position;
            this.canvas.apply_zoom(delta, f32::from(pos.x) as f64, f32::from(pos.y) as f64);
            cx.notify();
        }))
        .on_drop::<LibraryDragItem>(move |item, _window, app_cx| {
            let name = item.name.clone();
            let lib_path = item.lib_path.clone();
            let symbol_name = item.symbol_name.clone();
            entity
                .update(app_cx, |this, cx| {
                    this.handle_library_drop(&name, &lib_path, &symbol_name, cx);
                })
                .ok();
        })
        .on_action(cx.listener(|this, _: &super::Undo, _window, cx| {
            this.handle_undo(cx);
        }))
        .on_action(cx.listener(|this, _: &super::Redo, _window, cx| {
            this.handle_redo(cx);
        }))
        .on_action(cx.listener(|this, _: &super::Delete, _window, cx| {
            this.handle_delete(cx);
        }))
        .on_action(cx.listener(|this, _: &super::Escape, _window, cx| {
            this.handle_escape(cx);
        }))
        .children(grid_children)
        .children(object_children)
        .children(wire_preview)
        .child(tool_hint)
        .child(coord_hint)
}

// ── Grid rendering ─────────────────────────────────────

fn build_grid_dots(
    pan: eda_core::Point2D,
    zoom: f64,
    grid_mm: f64,
    cx_half_w: f64,
    cx_half_h: f64,
) -> Vec<AnyElement> {
    let mut dots = Vec::new();
    let grid_px = grid_mm * zoom;

    if !(8.0..=200.0).contains(&grid_px) {
        return dots;
    }

    let left_mm = pan.x - cx_half_w / zoom;
    let right_mm = pan.x + cx_half_w / zoom;
    let top_mm = pan.y - cx_half_h / zoom;
    let bottom_mm = pan.y + cx_half_h / zoom;

    let start_x = (left_mm / grid_mm).floor() as i64;
    let end_x = (right_mm / grid_mm).ceil() as i64;
    let start_y = (top_mm / grid_mm).floor() as i64;
    let end_y = (bottom_mm / grid_mm).ceil() as i64;

    let max_dots = 2000;
    let total = (end_x - start_x + 1) * (end_y - start_y + 1);
    let step = if total > max_dots {
        ((total as f64 / max_dots as f64).ceil() as i64).max(2)
    } else {
        1
    };

    let mut ix = start_x;
    while ix <= end_x {
        let mut iy = start_y;
        while iy <= end_y {
            let sx = (ix as f64 * grid_mm - pan.x) * zoom + cx_half_w;
            let sy = (iy as f64 * grid_mm - pan.y) * zoom + cx_half_h;

            let is_major = ix % 10 == 0 && iy % 10 == 0;
            let dot_size = if is_major { 3.0 } else { 1.0 };
            let color = if is_major {
                theme::BORDER_DEFAULT
            } else {
                theme::BORDER_SUBTLE
            };

            dots.push(
                div()
                    .absolute()
                    .left(px(sx as f32 - dot_size / 2.0))
                    .top(px(sy as f32 - dot_size / 2.0))
                    .w(px(dot_size))
                    .h(px(dot_size))
                    .rounded(px(dot_size))
                    .bg(color)
                    .into_any_element(),
            );

            iy += step;
        }
        ix += step;
    }

    dots
}

// ── Object rendering ───────────────────────────────────

struct RenderedObject {
    kind: SchematicObjectKind,
    name: String,
    pos: eda_core::Point2D,
    wire_seg: Option<eda_core::WireSegment>,
    selected: bool,
    graphics: Option<eda_core::SymbolGraphics>,
}

fn render_object(
    obj: &RenderedObject,
    pan: eda_core::Point2D,
    zoom: f64,
    cx_half_w: f64,
    cx_half_h: f64,
) -> AnyElement {
    let sx = (obj.pos.x - pan.x) * zoom + cx_half_w;
    let sy = (obj.pos.y - pan.y) * zoom + cx_half_h;

    let sel_border = if obj.selected {
        theme::ACCENT
    } else {
        gpui::Hsla::transparent_black()
    };

    match obj.kind {
        SchematicObjectKind::Symbol => {
            if let Some(ref gfx) = obj.graphics {
                let vp = ViewportParams { pan, zoom, half_w: cx_half_w, half_h: cx_half_h };
                render_symbol_graphics(gfx, &obj.name, obj.pos, &vp, sel_border)
            } else {
                render_symbol_placeholder(&obj.name, sx, sy, zoom, sel_border)
            }
        }
        SchematicObjectKind::Wire => {
            if let Some(seg) = &obj.wire_seg {
                render_wire(seg, pan, zoom, cx_half_w, cx_half_h, sel_border)
            } else {
                div().into_any_element()
            }
        }
        SchematicObjectKind::Label => render_label(&obj.name, sx, sy, sel_border),
        SchematicObjectKind::Junction => render_junction(sx, sy, zoom, sel_border),
    }
}

/// Fallback placeholder for symbols without parsed graphics.
fn render_symbol_placeholder(
    name: &str,
    sx: f64,
    sy: f64,
    zoom: f64,
    sel_border: gpui::Hsla,
) -> AnyElement {
    let w = (10.0 * zoom).clamp(40.0, 120.0);
    let h = (10.0 * zoom).clamp(28.0, 80.0);

    div()
        .absolute()
        .left(px(sx as f32 - w as f32 / 2.0))
        .top(px(sy as f32 - h as f32 / 2.0))
        .w(px(w as f32))
        .h(px(h as f32))
        .rounded(px(4.0))
        .border_2()
        .border_color(sel_border)
        .bg(theme::BG_ELEVATED)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .child(
            div()
                .text_color(theme::ACCENT)
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::BOLD)
                .child("◫"),
        )
        .child(
            div()
                .text_color(theme::TEXT_PRIMARY)
                .text_size(px(10.0))
                .child(name.to_string()),
        )
        .into_any_element()
}

/// Viewport parameters used during rendering.
struct ViewportParams {
    pan: eda_core::Point2D,
    zoom: f64,
    half_w: f64,
    half_h: f64,
}

/// Render a symbol with actual KiCad vector graphics.
fn render_symbol_graphics(
    gfx: &eda_core::SymbolGraphics,
    name: &str,
    world_pos: eda_core::Point2D,
    vp: &ViewportParams,
    sel_border: gpui::Hsla,
) -> AnyElement {
    let zoom = vp.zoom;
    // The symbol origin in screen space
    let origin_sx = (world_pos.x - vp.pan.x) * zoom + vp.half_w;
    let origin_sy = (world_pos.y - vp.pan.y) * zoom + vp.half_h;

    let body = gfx.body_bounds();
    let pad_mm = 1.5;
    let container_w = (body.width() + pad_mm * 2.0) * zoom;
    let container_h = (body.height() + pad_mm * 2.0) * zoom;
    let container_left = origin_sx + (body.min.x - pad_mm) * zoom;
    let container_top = origin_sy + (body.min.y - pad_mm) * zoom;

    let mut children: Vec<AnyElement> = Vec::new();

    // Render rectangles (symbol body)
    for rect in &gfx.rectangles {
        let rx = (rect.top_left().x - body.min.x + pad_mm) * zoom;
        let ry = (rect.top_left().y - body.min.y + pad_mm) * zoom;
        let rw = rect.width() * zoom;
        let rh = rect.height() * zoom;

        children.push(
            div()
                .absolute()
                .left(px(rx as f32))
                .top(px(ry as f32))
                .w(px(rw as f32))
                .h(px(rh as f32))
                .border_1()
                .border_color(theme::ACCENT)
                .bg(theme::BG_ELEVATED)
                .into_any_element(),
        );
    }

    // Render polylines
    for polyline in &gfx.polylines {
        for segment in polyline.points.windows(2) {
            let p1 = &segment[0];
            let p2 = &segment[1];
            let x1 = (p1.x - body.min.x + pad_mm) * zoom;
            let y1 = (p1.y - body.min.y + pad_mm) * zoom;
            let x2 = (p2.x - body.min.x + pad_mm) * zoom;
            let y2 = (p2.y - body.min.y + pad_mm) * zoom;

            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            if len < 0.5 {
                continue;
            }

            // Approximate lines with thin divs (horizontal or vertical preference)
            if dx.abs() >= dy.abs() {
                let left = x1.min(x2);
                let mid_y = (y1 + y2) / 2.0;
                children.push(
                    div()
                        .absolute()
                        .left(px(left as f32))
                        .top(px(mid_y as f32 - 1.0))
                        .w(px(dx.abs() as f32))
                        .h(px(2.0))
                        .bg(theme::ACCENT)
                        .into_any_element(),
                );
            } else {
                let top = y1.min(y2);
                let mid_x = (x1 + x2) / 2.0;
                children.push(
                    div()
                        .absolute()
                        .left(px(mid_x as f32 - 1.0))
                        .top(px(top as f32))
                        .w(px(2.0))
                        .h(px(dy.abs() as f32))
                        .bg(theme::ACCENT)
                        .into_any_element(),
                );
            }
        }
    }

    // Render circles
    for circle in &gfx.circles {
        let cx_local = (circle.center.x - body.min.x + pad_mm) * zoom;
        let cy_local = (circle.center.y - body.min.y + pad_mm) * zoom;
        let r = circle.radius * zoom;
        let diameter = r * 2.0;

        children.push(
            div()
                .absolute()
                .left(px((cx_local - r) as f32))
                .top(px((cy_local - r) as f32))
                .w(px(diameter as f32))
                .h(px(diameter as f32))
                .rounded(px(r as f32))
                .border_1()
                .border_color(theme::ACCENT)
                .into_any_element(),
        );
    }

    // Render pins
    for pin in &gfx.pins {
        let px_pos = (pin.position.x - body.min.x + pad_mm) * zoom;
        let py_pos = (pin.position.y - body.min.y + pad_mm) * zoom;
        let stub = pin.stub_end();
        let stub_px = (stub.x - body.min.x + pad_mm) * zoom;
        let stub_py = (stub.y - body.min.y + pad_mm) * zoom;

        // Pin line
        let dx = stub_px - px_pos;
        let dy = stub_py - py_pos;
        let len = (dx * dx + dy * dy).sqrt();
        if len >= 0.5 {
            if dx.abs() >= dy.abs() {
                let left = px_pos.min(stub_px);
                let mid_y = (py_pos + stub_py) / 2.0;
                children.push(
                    div()
                        .absolute()
                        .left(px(left as f32))
                        .top(px(mid_y as f32 - 0.5))
                        .w(px(dx.abs() as f32))
                        .h(px(1.0))
                        .bg(theme::SUCCESS)
                        .into_any_element(),
                );
            } else {
                let top = py_pos.min(stub_py);
                let mid_x = (px_pos + stub_px) / 2.0;
                children.push(
                    div()
                        .absolute()
                        .left(px(mid_x as f32 - 0.5))
                        .top(px(top as f32))
                        .w(px(1.0))
                        .h(px(dy.abs() as f32))
                        .bg(theme::SUCCESS)
                        .into_any_element(),
                );
            }
        }

        // Pin connection dot
        children.push(
            div()
                .absolute()
                .left(px(px_pos as f32 - 2.0))
                .top(px(py_pos as f32 - 2.0))
                .w(px(4.0))
                .h(px(4.0))
                .rounded(px(4.0))
                .bg(theme::SUCCESS)
                .into_any_element(),
        );

        // Pin number label (only when zoomed in enough)
        if zoom > 4.0 && !pin.number.is_empty() {
            let label_x = (px_pos + stub_px) / 2.0;
            let label_y = (py_pos + stub_py) / 2.0 - 8.0;
            children.push(
                div()
                    .absolute()
                    .left(px(label_x as f32 - 6.0))
                    .top(px(label_y as f32))
                    .text_color(theme::TEXT_MUTED)
                    .text_size(px(8.0))
                    .child(pin.number.clone())
                    .into_any_element(),
            );
        }
    }

    // Reference + value labels below the body
    if zoom > 3.0 {
        let label_y = (body.height() + pad_mm * 2.0) * zoom + 2.0;
        let label_x = pad_mm * zoom;

        let ref_text = if !gfx.reference.is_empty() {
            format!("{} — {}", gfx.reference, name)
        } else {
            name.to_string()
        };

        children.push(
            div()
                .absolute()
                .left(px(label_x as f32))
                .top(px(label_y as f32))
                .text_color(theme::TEXT_PRIMARY)
                .text_size(px(9.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(ref_text)
                .into_any_element(),
        );

        if !gfx.value.is_empty() {
            children.push(
                div()
                    .absolute()
                    .left(px(label_x as f32))
                    .top(px((label_y + 11.0) as f32))
                    .text_color(theme::TEXT_MUTED)
                    .text_size(px(8.0))
                    .child(gfx.value.clone())
                    .into_any_element(),
            );
        }
    }

    // Selection border around entire symbol
    let border_color = if sel_border.a > 0.0 {
        sel_border
    } else {
        gpui::Hsla::transparent_black()
    };

    div()
        .absolute()
        .left(px(container_left as f32))
        .top(px(container_top as f32))
        .w(px(container_w as f32))
        .h(px((container_h + if zoom > 3.0 { 24.0 } else { 0.0 }) as f32))
        .border_1()
        .border_color(border_color)
        .rounded(px(2.0))
        .relative()
        .children(children)
        .into_any_element()
}

fn render_wire(
    seg: &eda_core::WireSegment,
    pan: eda_core::Point2D,
    zoom: f64,
    cx_half_w: f64,
    cx_half_h: f64,
    sel_border: gpui::Hsla,
) -> AnyElement {
    let sx1 = (seg.start.x - pan.x) * zoom + cx_half_w;
    let sy1 = (seg.start.y - pan.y) * zoom + cx_half_h;
    let sx2 = (seg.end.x - pan.x) * zoom + cx_half_w;
    let sy2 = (seg.end.y - pan.y) * zoom + cx_half_h;

    let dx = sx2 - sx1;
    let dy = sy2 - sy1;
    let length = (dx * dx + dy * dy).sqrt();
    let thickness = 2.0_f32;

    let wire_color = if sel_border.a > 0.0 {
        theme::ACCENT
    } else {
        theme::SUCCESS
    };

    if length < 1.0 {
        return div().into_any_element();
    }

    if dx.abs() > dy.abs() {
        let left = sx1.min(sx2);
        div()
            .absolute()
            .left(px(left as f32))
            .top(px(sy1 as f32 - thickness / 2.0))
            .w(px(dx.abs() as f32))
            .h(px(thickness))
            .bg(wire_color)
            .into_any_element()
    } else {
        let top = sy1.min(sy2);
        div()
            .absolute()
            .left(px(sx1 as f32 - thickness / 2.0))
            .top(px(top as f32))
            .w(px(thickness))
            .h(px(dy.abs() as f32))
            .bg(wire_color)
            .into_any_element()
    }
}

fn render_wire_preview(
    start: eda_core::Point2D,
    end: eda_core::Point2D,
    pan: eda_core::Point2D,
    zoom: f64,
    cx_half_w: f64,
    cx_half_h: f64,
) -> AnyElement {
    let seg = eda_core::WireSegment::new(start, end);
    render_wire(&seg, pan, zoom, cx_half_w, cx_half_h, theme::WARNING)
}

fn render_label(name: &str, sx: f64, sy: f64, sel_border: gpui::Hsla) -> AnyElement {
    div()
        .absolute()
        .left(px(sx as f32 - 4.0))
        .top(px(sy as f32 - 10.0))
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .border_1()
        .border_color(sel_border)
        .bg(theme::BG_SURFACE)
        .text_color(theme::WARNING)
        .text_size(px(11.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .child(name.to_string())
        .into_any_element()
}

fn render_junction(sx: f64, sy: f64, zoom: f64, sel_border: gpui::Hsla) -> AnyElement {
    let size = (2.0 * zoom).clamp(6.0, 12.0);

    div()
        .absolute()
        .left(px(sx as f32 - size as f32 / 2.0))
        .top(px(sy as f32 - size as f32 / 2.0))
        .w(px(size as f32))
        .h(px(size as f32))
        .rounded(px(size as f32))
        .border_1()
        .border_color(sel_border)
        .bg(theme::SUCCESS)
        .into_any_element()
}
