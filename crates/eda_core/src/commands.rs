use crate::geometry::Point2D;
use crate::{DocumentError, EntityId, ProjectDocument, SchematicObject};

#[derive(Debug, Clone, PartialEq)]
pub enum EditorCommand {
    PlaceObject { object: SchematicObject },
    DeleteObject { id: EntityId },
    MoveObject { id: EntityId, new_pos: Point2D, old_pos: Point2D },
    ReplaceSelection { ids: Vec<EntityId> },
    ClearSelection,
}

#[derive(Debug, Clone, PartialEq)]
struct ExecutedCommand {
    forward: EditorCommand,
    inverse: EditorCommand,
}

#[derive(Debug, Clone, Default)]
pub struct CommandStack {
    undo: Vec<ExecutedCommand>,
    redo: Vec<ExecutedCommand>,
}

impl CommandStack {
    pub fn apply(
        &mut self,
        document: &mut ProjectDocument,
        command: EditorCommand,
    ) -> Result<(), DocumentError> {
        let inverse = document.apply_command(&command)?;
        self.undo.push(ExecutedCommand {
            forward: command,
            inverse,
        });
        self.redo.clear();
        Ok(())
    }

    pub fn undo(&mut self, document: &mut ProjectDocument) -> Result<bool, DocumentError> {
        let Some(executed) = self.undo.pop() else {
            return Ok(false);
        };

        let redo_inverse = document.apply_command(&executed.inverse)?;
        self.redo.push(ExecutedCommand {
            forward: executed.forward,
            inverse: redo_inverse,
        });
        Ok(true)
    }

    pub fn redo(&mut self, document: &mut ProjectDocument) -> Result<bool, DocumentError> {
        let Some(executed) = self.redo.pop() else {
            return Ok(false);
        };

        let undo_inverse = document.apply_command(&executed.forward)?;
        self.undo.push(ExecutedCommand {
            forward: executed.forward,
            inverse: undo_inverse,
        });
        Ok(true)
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

impl ProjectDocument {
    fn apply_command(&mut self, command: &EditorCommand) -> Result<EditorCommand, DocumentError> {
        match command {
            EditorCommand::PlaceObject { object } => {
                self.insert_object(object.clone())?;
                Ok(EditorCommand::DeleteObject { id: object.id() })
            }
            EditorCommand::DeleteObject { id } => {
                let removed = self.remove_object(*id)?;
                Ok(EditorCommand::PlaceObject { object: removed })
            }
            EditorCommand::MoveObject { id, new_pos, old_pos } => {
                let obj = self.objects_mut().iter_mut().find(|o| o.id() == *id)
                    .ok_or(DocumentError::MissingEntity(*id))?;
                obj.set_position(*new_pos);
                Ok(EditorCommand::MoveObject { id: *id, new_pos: *old_pos, old_pos: *new_pos })
            }
            EditorCommand::ReplaceSelection { ids } => {
                let previous = self.selection().iter().collect();
                self.replace_selection(ids.iter().copied())?;
                Ok(EditorCommand::ReplaceSelection { ids: previous })
            }
            EditorCommand::ClearSelection => {
                let previous = self.selection().iter().collect();
                self.clear_selection();
                Ok(EditorCommand::ReplaceSelection { ids: previous })
            }
        }
    }
}
