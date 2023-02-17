use egui;
use std::f64::consts::TAU;

use crate::svg_reader::Lines;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub(crate) struct Viewer {
    #[serde(skip)]
    lines: Lines,
    // this how you opt-out of serialization of a member
    //    #[serde(skip)]
    //    value: f32,
}

// impl Default for Viewer {
//     fn default() -> Self {
//         Self {
//             // Example stuff:
//             label: "Hello World!".to_owned(),
//             value: 2.7,
//         }
//     }
// }

impl Viewer {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>, lines: Lines) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        /*
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }*/

        Viewer { lines }
    }
}

impl eframe::App for Viewer {
    /// Called by the frame work to save state before shutdown.
    /*fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }*/

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        let frame =
            egui::Frame::central_panel(&ctx.style()).inner_margin(egui::style::Margin::same(0.));
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            let mut plot = egui::plot::Plot::new("svg_plot").data_aspect(1.0);

            let grid = true;
            if !grid {
                plot = plot.x_grid_spacer(|_| vec![]).y_grid_spacer(|_| vec![]);
            }

            plot.show(ui, |plot_ui| {
                // let n = 512;
                // let circle_points: egui::plot::PlotPoints = (0..=n)
                //     .map(|i| {
                //         let t = egui::emath::remap(i as f64, 0.0..=(n as f64), 0.0..=TAU);
                //         let r = 50.;
                //         [r * t.cos() + 10. as f64, r * t.sin() + 0. as f64]
                //     })
                //     .collect();

                for line in self.lines.lines.iter() {
                    plot_ui.line(
                        egui::plot::Line::new(egui::plot::PlotPoints::new(line.clone()))
                            .color(egui::ecolor::Color32::from_rgb(100, 200, 100))
                            .name("circle"),
                    );
                }
            });

            egui::warn_if_debug_build(ui);
        });
    }
}
