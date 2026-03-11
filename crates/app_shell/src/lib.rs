#![forbid(unsafe_code)]

use eda_core::{ProjectManifest, WorkspaceKind};

const DEFAULT_TOOL_SECTIONS: [&str; 4] = [
    "Workspace Switcher",
    "Tool Rail",
    "Inspector",
    "Command Palette",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSnapshot {
    pub project_name: String,
    pub active_workspace: WorkspaceKind,
    pub tool_sections: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct AppShell {
    manifest: ProjectManifest,
    active_workspace: WorkspaceKind,
    tool_sections: Vec<&'static str>,
}

impl AppShell {
    #[must_use]
    pub fn new(manifest: ProjectManifest) -> Self {
        let active_workspace = manifest.default_workspace();

        Self {
            manifest,
            active_workspace,
            tool_sections: DEFAULT_TOOL_SECTIONS.to_vec(),
        }
    }

    #[must_use]
    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    #[must_use]
    pub const fn active_workspace(&self) -> WorkspaceKind {
        self.active_workspace
    }

    pub fn activate_workspace(&mut self, workspace: WorkspaceKind) {
        self.active_workspace = workspace;
    }

    #[must_use]
    pub fn snapshot(&self) -> ShellSnapshot {
        ShellSnapshot {
            project_name: self.manifest.name().to_owned(),
            active_workspace: self.active_workspace,
            tool_sections: self.tool_sections.clone(),
        }
    }

    #[must_use]
    pub fn startup_banner(&self) -> String {
        format!(
            "{} — {} workspace ready",
            self.manifest.name(),
            self.active_workspace.label()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::AppShell;
    use eda_core::{ProjectManifest, WorkspaceKind};

    #[test]
    fn shell_starts_in_sketch_workspace() {
        let shell = AppShell::new(ProjectManifest::new("Starter Project"));
        let snapshot = shell.snapshot();

        assert_eq!(snapshot.project_name, "Starter Project");
        assert_eq!(snapshot.active_workspace, WorkspaceKind::Sketch);
        assert!(snapshot.tool_sections.contains(&"Command Palette"));
    }

    #[test]
    fn shell_switches_workspaces_without_rebuilding_manifest() {
        let mut shell = AppShell::new(ProjectManifest::new("Starter Project"));
        shell.activate_workspace(WorkspaceKind::Library);

        assert_eq!(shell.active_workspace(), WorkspaceKind::Library);
        assert_eq!(shell.manifest().name(), "Starter Project");
    }
}
