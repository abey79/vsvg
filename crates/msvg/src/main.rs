#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod multi_viewer;

use crate::multi_viewer::MultiViewer;
use std::error::Error;
use std::path::PathBuf;

fn show(paths: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "vsvg multi",
        native_options,
        Box::new(move |cc| {
            let style = egui::Style {
                visuals: egui::Visuals::light(),
                ..egui::Style::default()
            };
            cc.egui_ctx.set_style(style);

            Box::new(MultiViewer::new(paths))
        }),
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let paths = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    show(paths)
}
