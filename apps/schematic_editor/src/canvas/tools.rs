/// The active tool determines how mouse events on the canvas are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasTool {
    #[default]
    Select,
    Place,
    Wire,
    Label,
    Move,
    Pan,
}

impl CanvasTool {
    pub const SKETCH_TOOLS: &[CanvasTool] = &[
        Self::Select,
        Self::Place,
        Self::Wire,
        Self::Label,
        Self::Move,
        Self::Pan,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Place => "Place",
            Self::Wire => "Wire",
            Self::Label => "Label",
            Self::Move => "Move",
            Self::Pan => "Pan",
        }
    }

    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Select => "⊹",
            Self::Place => "◫",
            Self::Wire => "╱",
            Self::Label => "𝐀",
            Self::Move => "✥",
            Self::Pan => "☰",
        }
    }
}
