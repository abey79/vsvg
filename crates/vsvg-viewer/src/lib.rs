#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::let_underscore_untyped)]

mod document_widget;
mod engine;
mod frame_history;
mod painters;
mod render_data;
pub mod viewer;
#[cfg(target_arch = "wasm32")]
pub mod web_handle;

use eframe::CreationContext;
use std::sync::Arc;
pub use viewer::Viewer;

pub use crate::document_widget::DocumentWidget;

/// Viewer app for [`show()`] and [`show_tolerance()`].
struct ShowViewerApp {
    document: Arc<vsvg::Document>,
    tolerance: f64,
}

impl ViewerApp for ShowViewerApp {
    fn setup(
        &mut self,
        _cc: &CreationContext,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        document_widget.set_tolerance(self.tolerance);
        document_widget.set_document(self.document.clone());
        Ok(())
    }
}

const DEFAULT_RENDERER_TOLERANCE: f64 = 0.01;

/// Show a document in a window.
///
/// For native use only.
#[cfg(not(target_arch = "wasm32"))]
pub fn show(document: Arc<vsvg::Document>) -> anyhow::Result<()> {
    show_tolerance(document, DEFAULT_RENDERER_TOLERANCE)
}

/// Show a document in a window with a custom renderer tolerance.
///
/// The renderer tolerance is used to convert curves into line segments before rendering. Smaller
/// values yield less visible artifacts but require more CPU to render.
///
/// For native use only.
#[cfg(not(target_arch = "wasm32"))]
pub fn show_tolerance(document: Arc<vsvg::Document>, tolerance: f64) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "vsvg-viewer",
        native_options,
        Box::new(move |cc| {
            let style = egui::Style {
                visuals: egui::Visuals::light(),
                ..egui::Style::default()
            };
            cc.egui_ctx.set_style(style);

            Box::new(
                Viewer::new(
                    cc,
                    Box::new(ShowViewerApp {
                        document: document.clone(),
                        tolerance,
                    }),
                )
                .expect("viewer requires wgpu backend"),
            )
        }),
    )?;

    Ok(())
}

/// Implement this trait to build a custom viewer app based on [`Viewer`].
pub trait ViewerApp {
    fn setup(
        &mut self,
        _cc: &eframe::CreationContext,
        _document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// Main update hook
    ///
    /// Create side panels for UI and/or update the document widget.
    fn update(
        &mut self,
        _ctx: &egui::Context,
        _document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// Hook to modify the native options before starting the app.
    #[cfg(not(target_arch = "wasm32"))]
    fn options(&self, _native_option: &mut eframe::NativeOptions) {}

    /// Window title
    fn title(&self) -> String {
        "vsvg ViewerApp".to_owned()
    }

    /// Hook to load persistent data.
    fn load(&mut self, _storage: &dyn eframe::Storage) {}

    /// Hook to save persistent data.
    fn save(&self, _storage: &mut dyn eframe::Storage) {}
}

/// Show a custom [`ViewerApp`].
///
/// For native use only.
#[cfg(not(target_arch = "wasm32"))]
pub fn show_with_viewer_app(viewer_app: impl ViewerApp + 'static) -> anyhow::Result<()> {
    vsvg::trace_function!();

    let viewer_app = Box::new(viewer_app);

    let mut native_options = eframe::NativeOptions::default();
    viewer_app.options(&mut native_options);

    eframe::run_native(
        viewer_app.title().as_str(),
        native_options,
        Box::new(move |cc| {
            let style = egui::Style {
                visuals: egui::Visuals::light(),
                ..egui::Style::default()
            };
            cc.egui_ctx.set_style(style);

            Box::new(Viewer::new(cc, viewer_app).expect("viewer requires wgpu backend"))
        }),
    )?;

    Ok(())
}
