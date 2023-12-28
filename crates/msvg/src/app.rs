use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

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

#[derive(serde::Deserialize, serde::Serialize)]
struct State {
    side_panel_open: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            side_panel_open: true,
        }
    }
}

#[derive(Default)]
struct FileNameOverlay {
    should_show: bool,
    hide_time: f64,
    file_name: String,
}

impl FileNameOverlay {
    pub fn show(&mut self, file_name: &str) {
        self.should_show = true;
        self.file_name = file_name.to_owned();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, rect: &egui::Rect) {
        if self.should_show {
            self.hide_time = ui.input(|i| i.time) + 2.0;
            self.should_show = false;
        }

        let margin = 30.0;
        let max_width = rect.width() - 2.0 * margin;

        let cur_time = ui.input(|i| i.time);
        let opacity = ui
            .ctx()
            .animate_bool("file_name_hide".into(), self.hide_time > cur_time);

        if self.hide_time > cur_time {
            ui.ctx()
                .request_repaint_after(Duration::from_secs_f64(self.hide_time - cur_time));
        }

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
                fill: ui.visuals().window_fill().gamma_multiply(opacity),
                inner_margin: Margin::symmetric(10.0, 7.0),
                rounding: egui::Rounding::same(10.0),
                ..Default::default()
            })
            .show(ui.ctx(), |ui| {
                ui.visuals_mut().widgets.noninteractive.fg_stroke.color = ui
                    .visuals_mut()
                    .widgets
                    .noninteractive
                    .fg_stroke
                    .color
                    .gamma_multiply(opacity);
                ui.add(egui::Label::new(&self.file_name).truncate(true))
            });
    }
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

    file_name_overlay: FileNameOverlay,

    /// The channel rx
    rx: std::sync::mpsc::Receiver<LoadingMessage>,

    /// Are loading messages still coming in?
    ///
    /// UI keeps refreshing until this is false.
    waiting_for_messages: bool,

    /// Persisted application state.
    state: State,
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
            file_name_overlay: FileNameOverlay::default(),
            rx,
            waiting_for_messages: true,
            state: State::default(),
        }
    }

    fn switch_document(&mut self, new_active_document: usize) {
        if new_active_document != self.active_document {
            self.active_document = new_active_document;
            self.document_dirty = true;
            self.scroll_to_selected_row = true;
            self.show_file_name_overlay();
        }
    }

    fn show_file_name_overlay(&mut self) {
        self.file_name_overlay.show(
            self.paths[self.active_document]
                .file_name()
                .unwrap_or("!!! no file name>"),
        );
    }
}

impl ViewerApp for App {
    fn setup(
        &mut self,
        _cc: &CreationContext,
        _document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        self.show_file_name_overlay();

        Ok(())
    }

    fn handle_input(&mut self, ctx: &Context, _document_widget: &mut DocumentWidget) {
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::L) {
                self.state.side_panel_open = !self.state.side_panel_open;
            }
        });

        let new_active_document = ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                return self.active_document.saturating_sub(1);
            }

            if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                && self.active_document < self.loaded_documents.len() - 1
            {
                return self.active_document.saturating_add(1);
            }

            self.active_document
        });

        self.switch_document(new_active_document);
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

                let mut list_item = vsvg_viewer::list_item::ListItem::new(file_name);

                match state {
                    LoadedDocument::Pending => list_item = list_item.weak(true).active(false),
                    LoadedDocument::Loading => list_item = list_item.subdued(true).active(false),
                    LoadedDocument::Loaded(_) => {}
                }

                if self.active_document == i {
                    list_item = list_item.selected(true);
                }

                let response = list_item.show(ui);

                if response.clicked() {
                    self.active_document = i;
                    self.document_dirty = true;
                }

                if self.scroll_to_selected_row && self.active_document == i {
                    ui.scroll_to_rect(response.rect, None);
                    self.scroll_to_selected_row = false;
                }
            }
        };

        // --- Side panel structure ---

        egui::SidePanel::right("right_panel")
            .default_width(200.)
            .frame(egui::Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show_animated(ctx, self.state.side_panel_open, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;

                // set the clip rectangle for using `vsvg_viewer::list_item::ListItem`
                // lots of hacks to get pixel right...
                let mut rect = ui.available_rect_before_wrap();
                rect.min.x += 1.0;
                rect.min.y -= 1.0;
                ui.set_clip_rect(rect);
                ui.add_space(-1.0);

                egui::Frame {
                    inner_margin: Margin::symmetric(2.0, 0.0),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        egui::Frame {
                            inner_margin: Margin::symmetric(6.0, 0.0),
                            ..Default::default()
                        }
                        .show(ui, content_ui);
                    });
                });
            });

        // --- Update the document widget if needed ---

        if self.document_dirty {
            if let Some(LoadedDocument::Loaded(document)) =
                self.loaded_documents.get(&self.paths[self.active_document])
            {
                document_widget.set_document(document.clone());
                self.document_dirty = false;
            }
        }

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
        self.file_name_overlay.ui(ui, &rect);

        Ok(())
    }

    fn title(&self) -> String {
        "msvg".to_owned()
    }

    fn load(&mut self, storage: &dyn Storage) {
        if let Some(app_state) = eframe::get_value(storage, "msvg-app-state") {
            self.state = app_state;
        }
    }

    fn save(&self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "msvg-app-state", &self.state);
    }
}
