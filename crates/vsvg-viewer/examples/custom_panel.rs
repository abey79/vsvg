//! This example demonstrates how the [`ViewerApp`] trait can be used to customize the viewer UI
//! and control the content of the [`DocumentWidget`].

use eframe::CreationContext;
use rand::Rng;
use std::sync::Arc;
use vsvg::{Document, DocumentTrait};
use vsvg_viewer::{show_with_viewer_app, DocumentWidget, ViewerApp};

struct SidePanelViewerApp {
    document: Document,
}

impl SidePanelViewerApp {
    pub fn new() -> Self {
        Self {
            document: Document::new_with_page_size(vsvg::PageSize::A6V),
        }
    }
}

impl ViewerApp for SidePanelViewerApp {
    fn setup(
        &mut self,
        _cc: &CreationContext,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        document_widget.set_document(Arc::new(self.document.clone()));

        Ok(())
    }

    fn show_panels(
        &mut self,
        ctx: &egui::Context,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        egui::SidePanel::right("right_panel")
            .default_width(200.)
            .show(ctx, |ui| {
                if ui.button("Add line").clicked() {
                    let mut rng = rand::thread_rng();

                    let (w, h) = self
                        .document
                        .metadata()
                        .page_size
                        .map(Into::into)
                        .unwrap_or((200.0, 200.0));

                    self.document.push_path(
                        1,
                        kurbo::Line::new(
                            (rng.gen_range(0.0..w), rng.gen_range(0.0..h)),
                            (rng.gen_range(0.0..w), rng.gen_range(0.0..h)),
                        ),
                    );

                    document_widget.set_document(Arc::new(self.document.clone()));
                }
            });

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    show_with_viewer_app(SidePanelViewerApp::new())
}
