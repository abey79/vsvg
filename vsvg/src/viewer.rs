use eframe::Frame;
use egui::plot::PlotUi;
use egui::Ui;
use std::collections::HashMap;
use std::error::Error;
use vsvg_core::flattened_layer::FlattenedLayer;
use vsvg_core::Transforms;
use vsvg_core::{document::FlattenedDocument, LayerID, PageSize};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub(crate) struct Viewer {
    /// polylines derived from the document
    #[serde(skip)]
    document: FlattenedDocument,

    /// control points derived from the document
    #[serde(skip)]
    control_points: FlattenedDocument,

    #[serde(skip)]
    page_size: Option<PageSize>,

    /// show points
    show_point: bool,

    /// show grid
    show_grid: bool,

    /// show control points
    show_control_points: bool,

    /// layer visibility
    #[serde(skip)]
    layer_visibility: HashMap<LayerID, bool>,
}

fn vsvg_to_egui_color(val: vsvg_core::Color) -> egui::ecolor::Color32 {
    egui::ecolor::Color32::from_rgba_unmultiplied(val.r, val.g, val.b, val.a)
}

impl Viewer {
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        document: FlattenedDocument,
        control_points: FlattenedDocument,
        page_size: Option<PageSize>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        /*
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }*/

        Viewer {
            document,
            control_points,
            page_size,
            show_point: false,
            show_grid: false,
            show_control_points: false,
            layer_visibility: HashMap::new(),
        }
    }

    fn plot_page_size(&mut self, plot_ui: &mut PlotUi) {
        if let Some(page_size) = self.page_size {
            let page_frame = vec![
                [0.0, 0.0],
                [page_size.w, 0.0],
                [page_size.w, -page_size.h],
                [0.0, -page_size.h],
            ];

            // shadow
            plot_ui.polygon(
                egui::plot::Polygon::new(
                    page_frame
                        .iter()
                        .map(|p| [p[0] + SHADOW_OFFSET, p[1] - SHADOW_OFFSET])
                        .collect::<egui::plot::PlotPoints>(),
                )
                .color(egui::Color32::from_rgb(180, 180, 180))
                .fill_alpha(1.),
            );

            // background
            plot_ui.polygon(
                egui::plot::Polygon::new(
                    page_frame
                        .iter()
                        .copied()
                        .collect::<egui::plot::PlotPoints>(),
                )
                .color(egui::Color32::WHITE)
                .fill_alpha(1.),
            );

            // frame
            plot_ui.polygon(
                egui::plot::Polygon::new(
                    page_frame.into_iter().collect::<egui::plot::PlotPoints>(),
                )
                .color(egui::Color32::from_rgb(128, 128, 128))
                .fill_alpha(0.0),
            );
        }
    }

    fn plot_control_points(&self, plot_ui: &mut PlotUi, lid: LayerID) {
        if let Some(control_points) = self.control_points.try_get(lid) {
            for path in &control_points.paths {
                plot_ui.line(
                    egui::plot::Line::new(
                        path.data
                            .iter()
                            .copied()
                            .collect::<egui::plot::PlotPoints>(),
                    )
                    .color(egui::Color32::GRAY)
                    .width(0.5),
                );

                plot_ui.points(
                    egui::plot::Points::new(
                        path.data
                            .iter()
                            .copied()
                            .collect::<egui::plot::PlotPoints>(),
                    )
                    .color(egui::Color32::DARK_GRAY)
                    .radius(1.5),
                );
            }
        }
    }

    fn plot_layer(&self, plot_ui: &mut PlotUi, layer: &FlattenedLayer) {
        for path in &layer.paths {
            plot_ui.line(
                egui::plot::Line::new(
                    path.data
                        .iter()
                        .copied()
                        .collect::<egui::plot::PlotPoints>(),
                )
                .color(vsvg_to_egui_color(path.color))
                .width(path.stroke_width as f32),
            );

            if self.show_point {
                plot_ui.points(
                    egui::plot::Points::new(
                        path.data
                            .iter()
                            .copied()
                            .collect::<egui::plot::PlotPoints>(),
                    )
                    .color(vsvg_to_egui_color(path.color))
                    .radius(path.stroke_width as f32 * 2.0),
                );
            }
        }
    }

    fn show_viewer(&mut self, ui: &mut Ui) {
        let mut plot = egui::plot::Plot::new("svg_plot")
            .data_aspect(1.0)
            .show_background(false)
            .auto_bounds_x()
            .auto_bounds_y();

        if !self.show_grid {
            plot = plot.x_grid_spacer(|_| vec![]).y_grid_spacer(|_| vec![]);
        }

        plot.show(ui, |plot_ui| {
            self.plot_page_size(plot_ui);

            for (lid, layer) in &self.document.layers {
                if !self.layer_visibility.get(lid).unwrap_or(&true) {
                    continue;
                }

                // draw control points
                if self.show_control_points {
                    self.plot_control_points(plot_ui, *lid);
                }

                self.plot_layer(plot_ui, layer);
            }
        });
    }

    fn menu_file(&self, frame: &mut Frame, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                frame.close();
            }
        });
    }

    fn menu_view(&mut self, ui: &mut Ui) {
        ui.menu_button("View", |ui| {
            ui.checkbox(&mut self.show_point, "Show points");
            ui.checkbox(&mut self.show_grid, "Show grid");
            ui.checkbox(&mut self.show_control_points, "Show control points");
        });
    }

    fn menu_layer(&mut self, ui: &mut Ui) {
        ui.menu_button("Layer", |ui| {
            for (lid, layer) in &self.document.layers {
                let visibility = self.layer_visibility.entry(*lid).or_insert(true);
                let mut label = format!("Layer {lid}");
                if !layer.name.is_empty() {
                    label.push_str(&format!(": {}", layer.name));
                }

                ui.checkbox(visibility, label);
            }
        });
    }
}

const SHADOW_OFFSET: f64 = 10.;

impl eframe::App for Viewer {
    /// Called by the framework to save state before shutdown.
    /*fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }*/

    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                self.menu_file(frame, ui);
                self.menu_view(ui);
                self.menu_layer(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        let panel_frame = egui::Frame::central_panel(&ctx.style())
            .inner_margin(egui::style::Margin::same(0.))
            .fill(egui::Color32::from_rgb(242, 242, 242));
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| self.show_viewer(ui));
    }
}

pub(crate) trait Show {
    fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>>;
}

impl Show for vsvg_core::Document {
    fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>> {
        let native_options = eframe::NativeOptions::default();
        let page_size = self.page_size;
        let mut polylines = self.flatten(tolerance);
        polylines.scale_non_uniform(1.0, -1.0);

        let mut control_points = self.control_points();
        control_points.scale_non_uniform(1.0, -1.0);

        eframe::run_native(
            "vsvg",
            native_options,
            Box::new(move |cc| {
                let style = egui::Style {
                    visuals: egui::Visuals::light(),
                    ..egui::Style::default()
                };
                cc.egui_ctx.set_style(style);
                Box::new(Viewer::new(cc, polylines, control_points, page_size))
            }),
        )?;

        Ok(())
    }
}
