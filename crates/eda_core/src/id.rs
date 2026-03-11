use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectId(u64);

impl ProjectId {
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "project:{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct EntityId(u64);

impl EntityId {
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "entity:{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct IdGenerator {
    next_project: u64,
    next_entity: u64,
}

impl IdGenerator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_project: 1,
            next_entity: 1,
        }
    }

    pub fn next_project_id(&mut self) -> ProjectId {
        let id = ProjectId::new(self.next_project);
        self.next_project += 1;
        id
    }

    pub fn next_entity_id(&mut self) -> EntityId {
        let id = EntityId::new(self.next_entity);
        self.next_entity += 1;
        id
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
