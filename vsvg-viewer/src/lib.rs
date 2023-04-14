#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod engine;
mod frame_history;
mod painters;
mod viewer;

use crate::engine::DocumentData;
use crate::viewer::Viewer;
use std::error::Error;
use std::sync::Arc;
use vsvg_core::Document;

pub fn show(document: &Document) -> Result<(), Box<dyn Error>> {
    let native_options = eframe::NativeOptions::default();
    let document_data = Arc::new(DocumentData::new(document));

    eframe::run_native(
        "vsvg",
        native_options,
        Box::new(move |cc| {
            let style = egui::Style {
                visuals: egui::Visuals::light(),
                ..egui::Style::default()
            };
            cc.egui_ctx.set_style(style);

            Box::new(Viewer::new(cc, document_data).expect("viewer requires wgpu backend"))
        }),
    )?;

    Ok(())
}
