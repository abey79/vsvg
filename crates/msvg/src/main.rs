#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use camino::Utf8PathBuf;
use eframe::CreationContext;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vsvg::Document;
use vsvg_viewer::{show_with_viewer_app, DocumentWidget, ViewerApp};

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

struct MsvgViewerApp {
    /// The list of paths to be loaded.
    paths: Vec<Utf8PathBuf>,

    /// The loaded documents.
    loaded_documents: BTreeMap<Utf8PathBuf, LoadedDocument>,

    /// The currently selected document.
    active_document: usize,

    /// The channel rx
    rx: std::sync::mpsc::Receiver<LoadingMessage>,

    /// Are loading messages still coming in?
    ///
    /// UI keeps refreshing until this is false.
    waiting_for_messages: bool,
}

impl MsvgViewerApp {
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
            rx,
            waiting_for_messages: true,
        }
    }
}

impl ViewerApp for MsvgViewerApp {
    fn setup(
        &mut self,
        _cc: &CreationContext,
        _document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &egui::Context,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        let mut document_dirty = false;

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
                        document_dirty = true;
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

        egui::SidePanel::right("right_panel")
            .default_width(200.)
            .show(ctx, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    ctx.input(|i| {
                        if i.key_pressed(egui::Key::ArrowUp) {
                            self.active_document = self.active_document.saturating_sub(1);
                            document_dirty = true;
                        }

                        if i.key_pressed(egui::Key::ArrowDown)
                            && self.active_document < self.loaded_documents.len() - 1
                        {
                            self.active_document = self.active_document.saturating_add(1);
                            document_dirty = true;
                        }
                    });

                    for (i, path) in self.paths.iter().enumerate() {
                        let Some(state) = self.loaded_documents.get(path) else {
                            continue;
                        };

                        let file_name = path.file_name().map(ToOwned::to_owned).unwrap_or_default();

                        match state {
                            LoadedDocument::Pending => {
                                ui.weak(file_name);
                            }

                            LoadedDocument::Loading => {
                                ui.horizontal(|ui| {
                                    ui.weak(file_name);
                                    ui.spinner();
                                });
                            }
                            LoadedDocument::Loaded(document) => {
                                if ui
                                    .selectable_label(self.active_document == i, file_name)
                                    .clicked()
                                {
                                    self.active_document = i;
                                    document_dirty = true;
                                }

                                if document_dirty && self.active_document == i {
                                    document_widget.set_document(document.clone());
                                    document_dirty = false;
                                }
                            }
                        }
                    }
                });
            });

        Ok(())
    }
}

fn visit_file(file: PathBuf, paths: &mut Vec<Utf8PathBuf>) -> anyhow::Result<()> {
    if file.extension() == Some("svg".as_ref()) {
        paths.push(file.try_into()?);
    }

    Ok(())
}

fn visit_dir(dir: &Path, paths: &mut Vec<Utf8PathBuf>) -> anyhow::Result<()> {
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, paths)?;
        } else {
            visit_file(path, paths)?;
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut svg_list = Vec::new();

    for path in std::env::args().skip(1).map(PathBuf::from) {
        if path.is_dir() {
            visit_dir(&path, &mut svg_list)?;
        } else {
            visit_file(path, &mut svg_list)?;
        }
    }

    show_with_viewer_app(MsvgViewerApp::from_paths(svg_list))
}
