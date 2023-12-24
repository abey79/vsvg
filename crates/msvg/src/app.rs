use std::collections::BTreeMap;
use std::sync::Arc;

use camino::Utf8PathBuf;
use eframe::{CreationContext, Storage};
use egui::{Context, Margin};

use vsvg::Document;
use vsvg_viewer::{DocumentWidget, ViewerApp};

enum LoadingMessage {
    Starting(Utf8PathBuf),
    Loaded(Utf8PathBuf, Arc<Document>),
    //TODO: error state
    Completed,
}

#[derive(Default, Debug)]
enum LoadedDocument {
    #[default]
    Pending,
    Loading,
    Loaded(Arc<Document>),
    // TODO: add error state: Error(Box<dyn Error + Send>),
    // TODO: Document::from_svg() should return an proper error type
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct AppState {
    side_panel_open: bool,
}

pub(crate) struct App {
    /// The list of paths to be loaded.
    paths: Vec<Utf8PathBuf>,

    /// The loaded documents.
    loaded_documents: BTreeMap<Utf8PathBuf, LoadedDocument>,

    /// The currently selected document.
    active_document: usize,

    /// Flag indicating if the `DocumentWidget` should be updated.
    document_dirty: bool,

    /// Flag indicating if we should scroll to the selected file in the side panel.
    scroll_to_selected_row: bool,

    /// The channel rx
    rx: std::sync::mpsc::Receiver<LoadingMessage>,

    /// Are loading messages still coming in?
    ///
    /// UI keeps refreshing until this is false.
    waiting_for_messages: bool,

    /// Persisted application state.
    app_state: AppState,
}

impl App {
    pub fn from_paths(paths: Vec<Utf8PathBuf>) -> Self {
        let loaded_documents: BTreeMap<_, _> = paths
            .into_iter()
            .map(|path| (path, LoadedDocument::Pending))
            .collect();

        // make sure they are in the same order
        let paths: Vec<_> = loaded_documents.keys().cloned().collect();

        let (sender, rx) = std::sync::mpsc::channel::<LoadingMessage>();

        paths.clone().into_iter().for_each(|path| {
            let sender = sender.clone();
            rayon::spawn_fifo(move || {
                sender.send(LoadingMessage::Starting(path.clone())).unwrap();
                let document = Document::from_svg(&path, false).unwrap();
                sender
                    .send(LoadingMessage::Loaded(path.clone(), Arc::new(document)))
                    .unwrap();
            });
        });
        rayon::spawn_fifo(move || {
            sender.send(LoadingMessage::Completed).unwrap();
        });

        Self {
            paths,
            loaded_documents,
            active_document: 0,
            document_dirty: true,
            scroll_to_selected_row: false,
            rx,
            waiting_for_messages: true,
            app_state: AppState::default(),
        }
    }
}

impl ViewerApp for App {
    fn setup(
        &mut self,
        _cc: &CreationContext,
        _document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_input(&mut self, ctx: &Context, _document_widget: &mut DocumentWidget) {
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::L) {
                self.app_state.side_panel_open = !self.app_state.side_panel_open;
            }

            if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                self.active_document = self.active_document.saturating_sub(1);
                self.document_dirty = true;
                self.scroll_to_selected_row = true;
            }

            if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                && self.active_document < self.loaded_documents.len() - 1
            {
                self.active_document = self.active_document.saturating_add(1);
                self.document_dirty = true;
                self.scroll_to_selected_row = true;
            }
        });
    }

    fn show_panels(
        &mut self,
        ctx: &Context,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        // --- Handle incoming message ---

        for msg in self.rx.try_iter() {
            match msg {
                LoadingMessage::Starting(path) => {
                    self.loaded_documents
                        .entry(path)
                        .and_modify(|state| *state = LoadedDocument::Loading);
                }
                LoadingMessage::Loaded(path, doc) => {
                    // find index into paths vec
                    if self.paths.iter().position(|p| p == &path) == Some(self.active_document) {
                        self.document_dirty = true;
                    }

                    self.loaded_documents
                        .entry(path)
                        .and_modify(|state| *state = LoadedDocument::Loaded(doc));
                }
                LoadingMessage::Completed => {
                    self.waiting_for_messages = false;
                }
            }
        }

        if self.waiting_for_messages {
            ctx.request_repaint();
        }

        // --- Side panel content UI ---

        let content_ui = |ui: &mut egui::Ui| {
            for (i, path) in self.paths.iter().enumerate() {
                let Some(state) = self.loaded_documents.get(path) else {
                    continue;
                };

                let file_name = path.file_name().map(ToOwned::to_owned).unwrap_or_default();

                let response = match state {
                    LoadedDocument::Pending => ui.weak(file_name),

                    LoadedDocument::Loading => {
                        ui.horizontal(|ui| {
                            ui.weak(file_name);
                            ui.spinner();
                        })
                        .response
                    }
                    LoadedDocument::Loaded(_) => {
                        let response = ui.selectable_label(self.active_document == i, file_name);
                        if response.clicked() {
                            self.active_document = i;
                            self.document_dirty = true;
                        }

                        response
                    }
                };

                if self.scroll_to_selected_row && self.active_document == i {
                    ui.scroll_to_rect(response.rect, None);
                    self.scroll_to_selected_row = false;
                }
            }

            if self.document_dirty {
                if let Some(LoadedDocument::Loaded(document)) =
                    self.loaded_documents.get(&self.paths[self.active_document])
                {
                    document_widget.set_document(document.clone());
                    self.document_dirty = false;
                }
            }
        };

        // --- Side panel structure ---

        egui::SidePanel::right("right_panel")
            .default_width(200.)
            .frame(egui::Frame {
                fill: ctx.style().visuals.panel_fill,
                inner_margin: Margin::same(2.0),
                ..Default::default()
            })
            .show_animated(ctx, self.app_state.side_panel_open, |ui| {
                // always show the handle
                ui.style_mut().spacing.scroll.dormant_handle_opacity = 0.6;

                egui::ScrollArea::both().show(ui, |ui| {
                    egui::Frame {
                        inner_margin: Margin::symmetric(6.0, 0.0),
                        ..Default::default()
                    }
                    .show(ui, content_ui);
                });
            });

        Ok(())
    }

    /// Hook to show the central panel.
    ///
    /// This is call after the wgpu render callback that displays the document.
    fn show_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        _document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        let rect = ui.available_rect_before_wrap();
        let margin = 30.0;
        let max_width = rect.width() - 2.0 * margin;
        egui::Window::new("file_path")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .movable(false)
            .interactable(false)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .fixed_pos(rect.center_bottom() - egui::vec2(0.0, 40.0))
            .max_width(max_width)
            .default_width(max_width)
            .min_width(max_width)
            .frame(egui::Frame {
                fill: ui.visuals().window_fill(),
                inner_margin: Margin::symmetric(10.0, 7.0),
                rounding: egui::Rounding::same(10.0),
                ..Default::default()
            })
            .show(ui.ctx(), |ui| {
                ui.add(
                    egui::Label::new(
                        self.paths[self.active_document]
                            .file_name()
                            .unwrap_or("!!! no file name>"),
                    )
                    .truncate(true),
                )
            });
        Ok(())
    }

    fn title(&self) -> String {
        "msvg".to_owned()
    }

    fn load(&mut self, storage: &dyn Storage) {
        if let Some(app_state) = eframe::get_value(storage, "msvg-app-state") {
            self.app_state = app_state;
        }
    }

    fn save(&self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "msvg-app-state", &self.app_state);
    }
}
