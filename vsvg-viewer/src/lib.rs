#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod engine;
mod frame_history;
mod painters;
mod viewer;

use crate::viewer::Viewer;
use std::error::Error;

pub trait Show {
    fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>>;
}

impl Show for vsvg_core::Document {
    fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>> {
        let native_options = eframe::NativeOptions::default();

        //TODO: this is Engine's implementation details
        let polylines = self.flatten(tolerance);
        let control_points = self.control_points();

        eframe::run_native(
            "vsvg",
            native_options,
            Box::new(move |cc| {
                let style = egui::Style {
                    visuals: egui::Visuals::light(),
                    ..egui::Style::default()
                };
                cc.egui_ctx.set_style(style);

                Box::new(
                    Viewer::new(cc, polylines, control_points)
                        .expect("viewer requires wgpu backend"),
                )
            }),
        )?;

        Ok(())
    }
}
