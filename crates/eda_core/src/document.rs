use std::{error::Error, fmt};

use crate::{EntityId, ProjectId, ProjectManifest, SelectionSet};
use crate::geometry::{BoundingBox, Point2D, SymbolGraphics, WireSegment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchematicObjectKind {
    Symbol,
    Wire,
    Label,
    Junction,
}

impl SchematicObjectKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Symbol => "symbol",
            Self::Wire => "wire",
            Self::Label => "label",
            Self::Junction => "junction",
        }
    }
}

/// A single object on the schematic canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct SchematicObject {
    id: EntityId,
    kind: SchematicObjectKind,
    display_name: String,
    position: Point2D,
    rotation_deg: f64,
    wire_segment: Option<WireSegment>,
    symbol_graphics: Option<SymbolGraphics>,
}

impl SchematicObject {
    #[must_use]
    pub fn new(id: EntityId, kind: SchematicObjectKind, display_name: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            display_name: display_name.into(),
            position: Point2D::zero(),
            rotation_deg: 0.0,
            wire_segment: None,
            symbol_graphics: None,
        }
    }

    /// Create a symbol at a specific position.
    #[must_use]
    pub fn symbol(id: EntityId, name: impl Into<String>, pos: Point2D) -> Self {
        Self {
            id,
            kind: SchematicObjectKind::Symbol,
            display_name: name.into(),
            position: pos,
            rotation_deg: 0.0,
            wire_segment: None,
            symbol_graphics: None,
        }
    }

    /// Create a symbol with parsed graphics.
    #[must_use]
    pub fn symbol_with_graphics(
        id: EntityId,
        name: impl Into<String>,
        pos: Point2D,
        graphics: SymbolGraphics,
    ) -> Self {
        Self {
            id,
            kind: SchematicObjectKind::Symbol,
            display_name: name.into(),
            position: pos,
            rotation_deg: 0.0,
            wire_segment: None,
            symbol_graphics: Some(graphics),
        }
    }

    /// Create a wire between two points.
    #[must_use]
    pub fn wire(id: EntityId, segment: WireSegment) -> Self {
        Self {
            id,
            kind: SchematicObjectKind::Wire,
            display_name: String::new(),
            position: segment.start,
            rotation_deg: 0.0,
            wire_segment: Some(segment),
            symbol_graphics: None,
        }
    }

    /// Create a label at a position.
    #[must_use]
    pub fn label(id: EntityId, text: impl Into<String>, pos: Point2D) -> Self {
        Self {
            id,
            kind: SchematicObjectKind::Label,
            display_name: text.into(),
            position: pos,
            rotation_deg: 0.0,
            wire_segment: None,
            symbol_graphics: None,
        }
    }

    /// Create a junction at a position.
    #[must_use]
    pub fn junction(id: EntityId, pos: Point2D) -> Self {
        Self {
            id,
            kind: SchematicObjectKind::Junction,
            display_name: String::new(),
            position: pos,
            rotation_deg: 0.0,
            wire_segment: None,
            symbol_graphics: None,
        }
    }

    #[must_use]
    pub const fn id(&self) -> EntityId {
        self.id
    }

    #[must_use]
    pub const fn kind(&self) -> SchematicObjectKind {
        self.kind
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[must_use]
    pub const fn position(&self) -> Point2D {
        self.position
    }

    pub fn set_position(&mut self, pos: Point2D) {
        self.position = pos;
        if let Some(ref mut seg) = self.wire_segment {
            let dx = pos.x - seg.start.x;
            let dy = pos.y - seg.start.y;
            seg.start = pos;
            seg.end = seg.end.offset(dx, dy);
        }
    }

    #[must_use]
    pub const fn rotation_deg(&self) -> f64 {
        self.rotation_deg
    }

    pub fn set_rotation_deg(&mut self, deg: f64) {
        self.rotation_deg = deg;
    }

    #[must_use]
    pub const fn wire_segment(&self) -> Option<&WireSegment> {
        self.wire_segment.as_ref()
    }

    #[must_use]
    pub fn symbol_graphics(&self) -> Option<&SymbolGraphics> {
        self.symbol_graphics.as_ref()
    }

    /// Approximate bounding box for hit-testing.
    #[must_use]
    pub fn bounds(&self) -> BoundingBox {
        match self.kind {
            SchematicObjectKind::Symbol => {
                if let Some(ref gfx) = self.symbol_graphics {
                    let body = gfx.body_bounds();
                    // Translate body bounds to world position, with padding for pins
                    let pad = 2.54;
                    BoundingBox::new(
                        Point2D::new(
                            self.position.x + body.min.x - pad,
                            self.position.y + body.min.y - pad,
                        ),
                        Point2D::new(
                            self.position.x + body.max.x + pad,
                            self.position.y + body.max.y + pad,
                        ),
                    )
                } else {
                    BoundingBox::around(self.position, 5.0, 5.0)
                }
            }
            SchematicObjectKind::Wire => {
                if let Some(seg) = &self.wire_segment {
                    BoundingBox::new(
                        Point2D::new(seg.start.x.min(seg.end.x) - 0.5, seg.start.y.min(seg.end.y) - 0.5),
                        Point2D::new(seg.start.x.max(seg.end.x) + 0.5, seg.start.y.max(seg.end.y) + 0.5),
                    )
                } else {
                    BoundingBox::around(self.position, 1.0, 1.0)
                }
            }
            SchematicObjectKind::Label => BoundingBox::around(self.position, 8.0, 2.5),
            SchematicObjectKind::Junction => BoundingBox::around(self.position, 1.0, 1.0),
        }
    }
}

// Eq is needed for the command stack comparison — compare by id
impl Eq for SchematicObject {}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    DuplicateEntity(EntityId),
    MissingEntity(EntityId),
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateEntity(id) => write!(f, "duplicate entity id: {id}"),
            Self::MissingEntity(id) => write!(f, "missing entity id: {id}"),
        }
    }
}

impl Error for DocumentError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDocument {
    project_id: ProjectId,
    manifest: ProjectManifest,
    objects: Vec<SchematicObject>,
    selection: SelectionSet,
}

impl ProjectDocument {
    #[must_use]
    pub fn new(project_id: ProjectId, manifest: ProjectManifest) -> Self {
        Self {
            project_id,
            manifest,
            objects: Vec::new(),
            selection: SelectionSet::default(),
        }
    }

    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    #[must_use]
    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    #[must_use]
    pub fn objects(&self) -> &[SchematicObject] {
        &self.objects
    }

    pub fn objects_mut(&mut self) -> &mut Vec<SchematicObject> {
        &mut self.objects
    }

    #[must_use]
    pub fn selection(&self) -> &SelectionSet {
        &self.selection
    }

    #[must_use]
    pub fn object(&self, id: EntityId) -> Option<&SchematicObject> {
        self.objects.iter().find(|object| object.id() == id)
    }

    pub fn insert_object(&mut self, object: SchematicObject) -> Result<(), DocumentError> {
        if self.object(object.id()).is_some() {
            return Err(DocumentError::DuplicateEntity(object.id()));
        }

        self.objects.push(object);
        Ok(())
    }

    pub fn remove_object(&mut self, id: EntityId) -> Result<SchematicObject, DocumentError> {
        let Some(index) = self.objects.iter().position(|object| object.id() == id) else {
            return Err(DocumentError::MissingEntity(id));
        };

        self.selection.remove(id);
        Ok(self.objects.remove(index))
    }

    pub fn replace_selection<I>(&mut self, ids: I) -> Result<(), DocumentError>
    where
        I: IntoIterator<Item = EntityId>,
    {
        let ids: Vec<_> = ids.into_iter().collect();

        if let Some(missing_id) = ids.iter().copied().find(|id| self.object(*id).is_none()) {
            return Err(DocumentError::MissingEntity(missing_id));
        }

        self.selection.replace(ids);
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }
}
