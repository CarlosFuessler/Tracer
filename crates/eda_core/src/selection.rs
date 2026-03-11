use std::collections::BTreeSet;

use crate::EntityId;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectionSet {
    ids: BTreeSet<EntityId>,
}

impl SelectionSet {
    pub fn replace<I>(&mut self, ids: I)
    where
        I: IntoIterator<Item = EntityId>,
    {
        self.ids = ids.into_iter().collect();
    }

    pub fn clear(&mut self) {
        self.ids.clear();
    }

    pub fn remove(&mut self, id: EntityId) -> bool {
        self.ids.remove(&id)
    }

    #[must_use]
    pub fn contains(&self, id: EntityId) -> bool {
        self.ids.contains(&id)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.ids.iter().copied()
    }
}
