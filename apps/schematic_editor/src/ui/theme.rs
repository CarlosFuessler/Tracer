use gpui::Hsla;

/// Shapr3D-inspired dark theme colors.
// Background layers (darkest to lightest)
pub const BG_BASE: Hsla = hsla(215, 0.30, 0.06, 1.0);
pub const BG_SURFACE: Hsla = hsla(215, 0.28, 0.09, 1.0);
pub const BG_ELEVATED: Hsla = hsla(215, 0.25, 0.12, 1.0);
pub const BG_HOVER: Hsla = hsla(215, 0.22, 0.16, 1.0);

// Borders
pub const BORDER_SUBTLE: Hsla = hsla(215, 0.18, 0.20, 1.0);
pub const BORDER_DEFAULT: Hsla = hsla(215, 0.15, 0.28, 1.0);

// Text
pub const TEXT_PRIMARY: Hsla = hsla(210, 0.15, 0.93, 1.0);
pub const TEXT_SECONDARY: Hsla = hsla(215, 0.10, 0.62, 1.0);
pub const TEXT_MUTED: Hsla = hsla(215, 0.08, 0.45, 1.0);

// Accent (blue)
pub const ACCENT: Hsla = hsla(217, 0.90, 0.55, 1.0);
#[allow(dead_code)]
pub const ACCENT_HOVER: Hsla = hsla(217, 0.85, 0.62, 1.0);
pub const ACCENT_MUTED: Hsla = hsla(217, 0.60, 0.25, 1.0);

// Canvas
pub const CANVAS_BG: Hsla = hsla(220, 0.35, 0.05, 1.0);
#[allow(dead_code)]
pub const CANVAS_GRID: Hsla = hsla(215, 0.20, 0.12, 0.6);
#[allow(dead_code)]
pub const CANVAS_CROSSHAIR: Hsla = hsla(217, 0.80, 0.60, 0.4);

// Status
pub const SUCCESS: Hsla = hsla(142, 0.70, 0.50, 1.0);
#[allow(dead_code)]
pub const WARNING: Hsla = hsla(38, 0.90, 0.55, 1.0);

const fn hsla(h: u16, s: f32, l: f32, a: f32) -> Hsla {
    Hsla {
        h: h as f32 / 360.0,
        s,
        l,
        a,
    }
}
