#![forbid(unsafe_code)]

mod app;
mod clipboard;
mod commands;
mod document;
pub mod geometry;
mod id;
mod selection;

pub use app::{
    APP_METADATA, AppMetadata, DEFAULT_WORKSPACE_SEQUENCE, ProjectManifest, WorkspaceKind,
};
pub use clipboard::ClipboardBuffer;
pub use commands::{CommandStack, EditorCommand};
pub use document::{DocumentError, ProjectDocument, SchematicObject, SchematicObjectKind};
pub use geometry::{
    BoundingBox, PinDirection, Point2D, SymbolCircle, SymbolGraphics, SymbolPin, SymbolPolyline,
    SymbolRect, WireSegment,
};
pub use id::{EntityId, IdGenerator, ProjectId};
pub use selection::SelectionSet;

#[cfg(test)]
mod tests {
    use super::{
        ClipboardBuffer, CommandStack, EditorCommand, IdGenerator, ProjectDocument,
        ProjectManifest, SchematicObject, SchematicObjectKind, WorkspaceKind,
    };

    #[test]
    fn manifest_uses_shapr_style_workspace_flow() {
        let manifest = ProjectManifest::new("Starter Project");

        assert_eq!(
            manifest.workspaces(),
            &[
                WorkspaceKind::Sketch,
                WorkspaceKind::Library,
                WorkspaceKind::Model,
            ]
        );
        assert_eq!(manifest.default_workspace(), WorkspaceKind::Sketch);
    }

    #[test]
    fn workspace_labels_are_stable() {
        assert_eq!(WorkspaceKind::Sketch.label(), "Sketch");
        assert_eq!(WorkspaceKind::Library.label(), "Library");
        assert_eq!(WorkspaceKind::Model.label(), "Model");
    }

    #[test]
    fn command_stack_supports_place_select_clipboard_and_undo() -> Result<(), super::DocumentError>
    {
        let mut ids = IdGenerator::default();
        let manifest = ProjectManifest::new("Starter Project");
        let mut document = ProjectDocument::new(ids.next_project_id(), manifest);
        let mut commands = CommandStack::default();

        let resistor =
            SchematicObject::new(ids.next_entity_id(), SchematicObjectKind::Symbol, "R1");
        let wire = SchematicObject::new(ids.next_entity_id(), SchematicObjectKind::Wire, "N$1");

        commands.apply(
            &mut document,
            EditorCommand::PlaceObject {
                object: resistor.clone(),
            },
        )?;
        commands.apply(
            &mut document,
            EditorCommand::PlaceObject {
                object: wire.clone(),
            },
        )?;
        commands.apply(
            &mut document,
            EditorCommand::ReplaceSelection {
                ids: vec![resistor.id()],
            },
        )?;

        let clipboard = ClipboardBuffer::capture(&document);

        assert_eq!(document.objects().len(), 2);
        assert_eq!(document.selection().len(), 1);
        assert_eq!(clipboard.objects().len(), 1);
        assert_eq!(clipboard.objects()[0].display_name(), "R1");

        assert!(commands.undo(&mut document)?);
        assert!(document.selection().is_empty());

        assert!(commands.undo(&mut document)?);
        assert_eq!(document.objects().len(), 1);

        assert!(commands.redo(&mut document)?);
        assert_eq!(document.objects().len(), 2);

        Ok(())
    }
}
