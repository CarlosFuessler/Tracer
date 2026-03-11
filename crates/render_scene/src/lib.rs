#![forbid(unsafe_code)]

use eda_core::WorkspaceKind;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneBootstrap {
    workspace: WorkspaceKind,
    zoom: f32,
    grid_step_mm: f32,
    snap_to_grid: bool,
}

impl SceneBootstrap {
    #[must_use]
    pub const fn for_workspace(workspace: WorkspaceKind) -> Self {
        let grid_step_mm = match workspace {
            WorkspaceKind::Sketch => 1.27,
            WorkspaceKind::Library => 0.635,
            WorkspaceKind::Model => 1.0,
        };

        Self {
            workspace,
            zoom: 1.0,
            grid_step_mm,
            snap_to_grid: true,
        }
    }

    #[must_use]
    pub const fn workspace(self) -> WorkspaceKind {
        self.workspace
    }

    #[must_use]
    pub const fn zoom(self) -> f32 {
        self.zoom
    }

    #[must_use]
    pub const fn grid_step_mm(self) -> f32 {
        self.grid_step_mm
    }

    #[must_use]
    pub const fn snap_to_grid(self) -> bool {
        self.snap_to_grid
    }

    #[must_use]
    pub fn summary(self) -> String {
        format!(
            "{} canvas @ {:.0}% zoom, {:.3} mm grid, snap {}",
            self.workspace.label(),
            self.zoom * 100.0,
            self.grid_step_mm,
            if self.snap_to_grid { "on" } else { "off" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SceneBootstrap;
    use eda_core::WorkspaceKind;

    #[test]
    fn sketch_workspace_prefers_symbol_friendly_grid() {
        let scene = SceneBootstrap::for_workspace(WorkspaceKind::Sketch);

        assert_eq!(scene.workspace(), WorkspaceKind::Sketch);
        assert_eq!(scene.grid_step_mm(), 1.27);
        assert!(scene.snap_to_grid());
    }

    #[test]
    fn summary_mentions_workspace_mode() {
        let summary = SceneBootstrap::for_workspace(WorkspaceKind::Library).summary();

        assert!(summary.contains("Library"));
        assert!(summary.contains("grid"));
    }
}
