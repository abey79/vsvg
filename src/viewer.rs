use crate::types::Polylines;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub(crate) struct Viewer {
    #[serde(skip)]
    polylines: Polylines,
}

#[allow(clippy::derivable_impls)]
impl Default for Viewer {
    // implementing default is needed for egui's persistence feature
    fn default() -> Self {
        Self {
            polylines: Polylines::default(),
        }
    }
}

// TODO: draw page size
// TODO: light mode
// TODO: line color & width
// TODO: API takes a `&Document`

impl Viewer {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>, polylines: Polylines) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        /*
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }*/

        Viewer { polylines }
    }
}

impl eframe::App for Viewer {
    /// Called by the frame work to save state before shutdown.
    /*fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }*/

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                        frame.close();
                    }
                });
                egui::warn_if_debug_build(ui);
            });
        });

        let panel_frame =
            egui::Frame::central_panel(&ctx.style()).inner_margin(egui::style::Margin::same(0.));
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                let mut plot = egui::plot::Plot::new("svg_plot").data_aspect(1.0);

                let grid = true;
                if !grid {
                    plot = plot.x_grid_spacer(|_| vec![]).y_grid_spacer(|_| vec![]);
                }

                plot.show(ui, |plot_ui| {
                    for line in self.polylines.iter() {
                        plot_ui.line(
                            egui::plot::Line::new(egui::plot::PlotPoints::new(line.points.clone()))
                                .color(egui::ecolor::Color32::from_rgb(100, 200, 100)),
                        );
                        plot_ui.points(
                            egui::plot::Points::new(egui::plot::PlotPoints::new(
                                line.points.clone(),
                            ))
                            .color(egui::ecolor::Color32::from_rgb(30, 200, 250))
                            .radius(2.0),
                        );
                    }
                });
            });
    }
}
