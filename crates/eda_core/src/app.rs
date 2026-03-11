#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WorkspaceKind {
    #[default]
    Sketch,
    Library,
    Model,
}

impl WorkspaceKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sketch => "Sketch",
            Self::Library => "Library",
            Self::Model => "Model",
        }
    }
}

pub const DEFAULT_WORKSPACE_SEQUENCE: &[WorkspaceKind; 3] = &[
    WorkspaceKind::Sketch,
    WorkspaceKind::Library,
    WorkspaceKind::Model,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppMetadata {
    pub product_name: &'static str,
    pub tagline: &'static str,
}

pub const APP_METADATA: AppMetadata = AppMetadata {
    product_name: "Tracer",
    tagline: "Modern KiCad-compatible EDA — inspired by Shapr3D",
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectManifest {
    name: String,
    workspaces: Vec<WorkspaceKind>,
}

impl ProjectManifest {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            workspaces: DEFAULT_WORKSPACE_SEQUENCE.to_vec(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn workspaces(&self) -> &[WorkspaceKind] {
        &self.workspaces
    }

    #[must_use]
    pub fn default_workspace(&self) -> WorkspaceKind {
        self.workspaces.first().copied().unwrap_or_default()
    }
}
