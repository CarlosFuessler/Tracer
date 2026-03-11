use crate::{ProjectDocument, SchematicObject};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClipboardBuffer {
    objects: Vec<SchematicObject>,
}

impl ClipboardBuffer {
    #[must_use]
    pub fn capture(document: &ProjectDocument) -> Self {
        let objects = document
            .selection()
            .iter()
            .filter_map(|id| document.object(id).cloned())
            .collect();

        Self { objects }
    }

    #[must_use]
    pub fn objects(&self) -> &[SchematicObject] {
        &self.objects
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}
