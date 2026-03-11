use eda_core::geometry::Point2D;

use super::CanvasTool;

/// Runtime state for the interactive schematic canvas.
#[derive(Debug, Clone)]
pub struct CanvasState {
    /// Active tool.
    pub tool: CanvasTool,
    /// Pan offset in schematic-space millimeters.
    pub pan: Point2D,
    /// Zoom factor (1.0 = 100%).
    pub zoom: f64,
    /// Grid step in mm.
    pub grid_mm: f64,
    /// Whether snap-to-grid is enabled.
    pub snap: bool,
    /// Mouse position in schematic space (updated on mouse move).
    pub mouse_schematic: Point2D,
    /// When drawing a wire, the start point of the current segment.
    pub wire_start: Option<Point2D>,
    /// Label text being placed (set via dialog before click).
    pub pending_label_text: Option<String>,
    /// Size of the canvas viewport in pixels (updated on render).
    pub viewport_px: (f64, f64),
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            tool: CanvasTool::Select,
            pan: Point2D::zero(),
            zoom: 8.0, // 8 px per mm gives a nice default for 1.27mm grid
            grid_mm: 1.27,
            snap: true,
            mouse_schematic: Point2D::zero(),
            wire_start: None,
            pending_label_text: None,
            viewport_px: (800.0, 600.0),
        }
    }
}

impl CanvasState {
    /// Convert schematic-space coordinates to screen pixels.
    #[must_use]
    pub fn to_screen(&self, pt: Point2D) -> (f64, f64) {
        let cx = self.viewport_px.0 / 2.0;
        let cy = self.viewport_px.1 / 2.0;
        let sx = (pt.x - self.pan.x) * self.zoom + cx;
        let sy = (pt.y - self.pan.y) * self.zoom + cy;
        (sx, sy)
    }

    /// Convert screen pixels to schematic-space coordinates.
    #[must_use]
    pub fn to_schematic(&self, screen_x: f64, screen_y: f64) -> Point2D {
        let cx = self.viewport_px.0 / 2.0;
        let cy = self.viewport_px.1 / 2.0;
        Point2D::new(
            (screen_x - cx) / self.zoom + self.pan.x,
            (screen_y - cy) / self.zoom + self.pan.y,
        )
    }

    /// Snap a schematic point to grid if snap is on.
    #[must_use]
    pub fn maybe_snap(&self, pt: Point2D) -> Point2D {
        if self.snap {
            pt.snapped(self.grid_mm)
        } else {
            pt
        }
    }

    /// Apply scroll-wheel zoom, centered on the given screen point.
    pub fn apply_zoom(&mut self, delta: f64, screen_x: f64, screen_y: f64) {
        let before = self.to_schematic(screen_x, screen_y);
        let factor = if delta > 0.0 { 1.15 } else { 1.0 / 1.15 };
        self.zoom = (self.zoom * factor).clamp(1.0, 200.0);
        let after = self.to_schematic(screen_x, screen_y);
        self.pan.x -= after.x - before.x;
        self.pan.y -= after.y - before.y;
    }

    /// Apply a pan delta in screen pixels.
    #[allow(dead_code)]
    pub fn apply_pan_px(&mut self, dx: f64, dy: f64) {
        self.pan.x -= dx / self.zoom;
        self.pan.y -= dy / self.zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_coordinates() {
        let state = CanvasState::default();
        let pt = Point2D::new(10.0, 20.0);
        let (sx, sy) = state.to_screen(pt);
        let back = state.to_schematic(sx, sy);
        assert!((back.x - pt.x).abs() < 0.001);
        assert!((back.y - pt.y).abs() < 0.001);
    }

    #[test]
    fn zoom_changes_scale() {
        let mut state = CanvasState::default();
        let old_zoom = state.zoom;
        state.apply_zoom(1.0, 400.0, 300.0);
        assert!(state.zoom > old_zoom);
    }
}
