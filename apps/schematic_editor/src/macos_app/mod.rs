use crate::EditorBootstrap;
use crate::canvas::{CanvasState, CanvasTool};
use eda_core::{
    EditorCommand, Point2D, SchematicObject, WireSegment,
};
use gpui::{
    App, AppContext, Application, Bounds, Context, KeyBinding, PathPromptOptions, Render, Window,
    WindowBounds, WindowOptions, actions, size,
};

mod browser;
mod canvas_view;
mod view;

actions!(schematic_editor, [Quit, Undo, Redo, Delete, Escape]);

pub(crate) fn run(bootstrap: EditorBootstrap) {
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(gpui::px(1400.0), gpui::px(920.0)), cx);

        cx.bind_keys([
            KeyBinding::new(quit_shortcut(), Quit, None),
            KeyBinding::new("cmd-z", Undo, None),
            KeyBinding::new("cmd-shift-z", Redo, None),
            KeyBinding::new("backspace", Delete, None),
            KeyBinding::new("escape", Escape, None),
        ]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        if let Err(error) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                window.set_window_title("Tracer");
                cx.new(|_| SchematicEditorWindow::new(bootstrap))
            },
        ) {
            eprintln!("failed to open main schematic editor window: {error}");
            cx.quit();
        }

        cx.activate(true);
    });
}

#[cfg(target_os = "macos")]
fn quit_shortcut() -> &'static str {
    "cmd-q"
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn quit_shortcut() -> &'static str {
    "ctrl-q"
}

#[derive(Debug, Clone)]
pub(crate) struct SchematicEditorWindow {
    pub(crate) bootstrap: EditorBootstrap,
    pub(crate) canvas: CanvasState,
    pub(crate) status_message: String,
    /// Counter for generating default symbol designators.
    pub(crate) place_counter: u32,
    /// Current component search query.
    pub(crate) search_query: String,
}

impl SchematicEditorWindow {
    fn new(bootstrap: EditorBootstrap) -> Self {
        Self {
            bootstrap,
            canvas: CanvasState::default(),
            status_message: "Ready — type to search components, drag onto canvas."
                .to_string(),
            place_counter: 0,
            search_query: String::new(),
        }
    }

    pub(crate) fn handle_library_drop(
        &mut self,
        name: &str,
        lib_path: &str,
        symbol_name: &str,
        cx: &mut Context<'_, Self>,
    ) {
        self.place_counter += 1;
        let pt = self.canvas.maybe_snap(self.canvas.mouse_schematic);
        let graphics = self.bootstrap.load_symbol_graphics(lib_path, symbol_name);
        let obj = if let Some(gfx) = graphics {
            SchematicObject::symbol_with_graphics(
                self.bootstrap.ids.next_entity_id(),
                name,
                pt,
                gfx,
            )
        } else {
            SchematicObject::symbol(self.bootstrap.ids.next_entity_id(), name, pt)
        };
        let _ = self.bootstrap.commands.apply(
            &mut self.bootstrap.document,
            EditorCommand::PlaceObject { object: obj },
        );
        self.status_message = format!("Placed {name} at {pt}");
        cx.notify();
    }

    pub(crate) fn set_tool(&mut self, tool: CanvasTool, cx: &mut Context<'_, Self>) {
        self.canvas.tool = tool;
        self.canvas.wire_start = None;
        self.status_message = format!("{} tool active", tool.label());
        cx.notify();
    }

    /// Handle a mouse click on the canvas at the given screen position.
    pub(crate) fn canvas_click(
        &mut self,
        screen_x: f64,
        screen_y: f64,
        cx: &mut Context<'_, Self>,
    ) {
        let raw = self.canvas.to_schematic(screen_x, screen_y);
        let pt = self.canvas.maybe_snap(raw);

        match self.canvas.tool {
            CanvasTool::Select => self.handle_select(pt, cx),
            CanvasTool::Place => self.handle_place(pt, cx),
            CanvasTool::Wire => self.handle_wire(pt, cx),
            CanvasTool::Label => self.handle_label(pt, cx),
            CanvasTool::Move | CanvasTool::Pan => {}
        }
    }

    fn handle_select(&mut self, pt: Point2D, cx: &mut Context<'_, Self>) {
        // Find the object closest to the click point within hit range
        let hit_range = 5.0 / self.canvas.zoom; // 5 screen pixels converted to mm
        let hit = self
            .bootstrap
            .document
            .objects()
            .iter()
            .find(|obj| obj.bounds().contains(pt) || obj.position().distance_to(pt) < hit_range);

        if let Some(obj) = hit {
            let id = obj.id();
            let name = obj.display_name().to_string();
            let _ = self.bootstrap.commands.apply(
                &mut self.bootstrap.document,
                EditorCommand::ReplaceSelection { ids: vec![id] },
            );
            self.status_message = format!("Selected: {}", name);
        } else {
            let _ = self.bootstrap.commands.apply(
                &mut self.bootstrap.document,
                EditorCommand::ClearSelection,
            );
            self.status_message = format!("Click at {pt}");
        }
        cx.notify();
    }

    fn handle_place(&mut self, pt: Point2D, cx: &mut Context<'_, Self>) {
        self.place_counter += 1;
        let name = format!("U{}", self.place_counter);
        let obj = SchematicObject::symbol(
            self.bootstrap.ids.next_entity_id(),
            &name,
            pt,
        );
        let _ = self.bootstrap.commands.apply(
            &mut self.bootstrap.document,
            EditorCommand::PlaceObject { object: obj },
        );
        self.status_message = format!("Placed {name} at {pt}");
        cx.notify();
    }

    fn handle_wire(&mut self, pt: Point2D, cx: &mut Context<'_, Self>) {
        if let Some(start) = self.canvas.wire_start.take() {
            // Complete wire
            let seg = WireSegment::new(start, pt);
            let obj = SchematicObject::wire(self.bootstrap.ids.next_entity_id(), seg);
            let _ = self.bootstrap.commands.apply(
                &mut self.bootstrap.document,
                EditorCommand::PlaceObject { object: obj },
            );
            self.status_message = format!("Wire from {start} to {pt}");
        } else {
            // Start wire
            self.canvas.wire_start = Some(pt);
            self.status_message = format!("Wire start at {pt} — click to finish");
        }
        cx.notify();
    }

    fn handle_label(&mut self, pt: Point2D, cx: &mut Context<'_, Self>) {
        let text = self
            .canvas
            .pending_label_text
            .take()
            .unwrap_or_else(|| format!("NET{}", self.bootstrap.document.objects().len()));
        let obj = SchematicObject::label(self.bootstrap.ids.next_entity_id(), &text, pt);
        let _ = self.bootstrap.commands.apply(
            &mut self.bootstrap.document,
            EditorCommand::PlaceObject { object: obj },
        );
        self.status_message = format!("Placed label \"{text}\" at {pt}");
        cx.notify();
    }

    pub(crate) fn handle_undo(&mut self, cx: &mut Context<'_, Self>) {
        let _ = self.bootstrap.commands.undo(&mut self.bootstrap.document);
        self.status_message = "Undo".to_string();
        cx.notify();
    }

    pub(crate) fn handle_redo(&mut self, cx: &mut Context<'_, Self>) {
        let _ = self.bootstrap.commands.redo(&mut self.bootstrap.document);
        self.status_message = "Redo".to_string();
        cx.notify();
    }

    pub(crate) fn handle_delete(&mut self, cx: &mut Context<'_, Self>) {
        let selected: Vec<_> = self.bootstrap.document.selection().iter().collect();
        for id in selected {
            let _ = self.bootstrap.commands.apply(
                &mut self.bootstrap.document,
                EditorCommand::DeleteObject { id },
            );
        }
        self.status_message = "Deleted selection".to_string();
        cx.notify();
    }

    pub(crate) fn handle_escape(&mut self, cx: &mut Context<'_, Self>) {
        self.canvas.wire_start = None;
        let _ = self.bootstrap.commands.apply(
            &mut self.bootstrap.document,
            EditorCommand::ClearSelection,
        );
        self.status_message = "Cancelled".to_string();
        cx.notify();
    }

    pub(crate) fn prompt_for_import(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.status_message = "Opening the KiCad import picker…".to_string();
        cx.notify();

        let entity = cx.weak_entity();
        let selection = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: true,
            multiple: true,
            prompt: Some("Choose KiCad schematics, symbol libraries, or folders".into()),
        });

        window
            .spawn(cx, async move |cx| {
                match selection.await {
                    Ok(Ok(Some(paths))) => {
                        let import_result = kicad_fmt::import_from_paths(&paths);

                        entity
                            .update_in(cx, |this, _window, cx| {
                                match import_result {
                                    Ok(imports) => {
                                        let imported_file_count = imports.documents().len();
                                        this.bootstrap.imports = imports;
                                        this.bootstrap.refresh_imported_libraries();
                                        this.status_message = if imported_file_count == 0 {
                                            "No KiCad files found in the selected locations.".to_string()
                                        } else {
                                            format!(
                                                "Imported {} KiCad file(s) from disk.",
                                                imported_file_count
                                            )
                                        };
                                    }
                                    Err(error) => {
                                        this.status_message = format!("Import failed: {error}");
                                    }
                                }

                                cx.notify();
                            })
                            .ok();
                    }
                    Ok(Ok(None)) => {
                        entity
                            .update(cx, |this, cx| {
                                this.status_message = "Import cancelled.".to_string();
                                cx.notify();
                            })
                            .ok();
                    }
                    Ok(Err(error)) => {
                        entity
                            .update(cx, |this, cx| {
                                this.status_message = format!("Import picker failed: {error}");
                                cx.notify();
                            })
                            .ok();
                    }
                    Err(error) => {
                        entity
                            .update(cx, |this, cx| {
                                this.status_message =
                                    format!("Import picker disconnected: {error}");
                                cx.notify();
                            })
                            .ok();
                    }
                }
            })
            .detach();
    }
}

impl Render for SchematicEditorWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui::IntoElement {
        view::render(self, cx)
    }
}
