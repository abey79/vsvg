use eframe::Frame;
use egui::plot::PlotUi;
use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};
use std::collections::HashMap;
use std::error::Error;
use vsvg_core::flattened_layer::FlattenedLayer;
use vsvg_core::{document::FlattenedDocument, LayerID, PageSize};
use vsvg_core::{FlattenedPath, Transforms};
use vsvg_viewer::triangulation::{build_fat_line, Triangle};

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

    /// show fat lines
    show_fat_lines: bool,

    /// show fat lines debug
    show_fat_lines_debug: bool,

    /// layer visibility
    #[serde(skip)]
    layer_visibility: HashMap<LayerID, bool>,

    /// pan offset
    ///
    /// The offset is expressed in SVG coordinates, not in pixels. `self.scale` can be used for
    /// conversion.
    #[serde(skip)]
    offset: Pos2,

    /// scale factor
    #[serde(skip)]
    scale: f32,

    /// Show settings window.
    show_settings: bool,

    /// Show inspection window.
    show_inspection: bool,

    /// Show memory window.
    show_memory: bool,
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
            show_fat_lines: true,
            show_fat_lines_debug: false,
            layer_visibility: HashMap::new(),
            offset: Pos2::ZERO,
            scale: 1.0,
            show_settings: false,
            show_inspection: false,
            show_memory: false,
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
                        .map(|p| [p[0] + SHADOW_OFFSET as f64, p[1] - SHADOW_OFFSET as f64])
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

    fn plot_layer_fat_lines(&self, plot_ui: &mut PlotUi, layer: &FlattenedLayer) {
        let mut vertices: Vec<kurbo::Point> = Vec::new(); //opt: pre-allocate
        let mut triangles: Vec<Triangle> = Vec::new(); //opt: pre-allocate

        for FlattenedPath {
            data: path,
            color: _color,
            stroke_width,
        } in &layer.paths
        {
            build_fat_line(path, *stroke_width, &mut vertices, &mut triangles);
        }

        // plot triangles
        for (i1, i2, i3) in triangles {
            plot_ui.polygon(
                egui::plot::Polygon::new(
                    [
                        [vertices[i1].x, vertices[i1].y],
                        [vertices[i2].x, vertices[i2].y],
                        [vertices[i3].x, vertices[i3].y],
                    ]
                    .iter()
                    .copied()
                    .collect::<egui::plot::PlotPoints>(),
                )
                .width(0.0)
                .fill_alpha(0.3),
            );
        }

        // plot debug stuff
        if self.show_fat_lines_debug {
            for FlattenedPath { data: path, .. } in &layer.paths {
                plot_ui.line(
                    egui::plot::Line::new(path.iter().copied().collect::<egui::plot::PlotPoints>())
                        .width(1.)
                        .color(egui::Color32::LIGHT_GREEN),
                );

                plot_ui.points(
                    egui::plot::Points::new(
                        path.iter().copied().collect::<egui::plot::PlotPoints>(),
                    )
                    .radius(3.)
                    .color(egui::Color32::LIGHT_GREEN),
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

                if self.show_fat_lines {
                    self.plot_layer_fat_lines(plot_ui, layer);
                } else {
                    self.plot_layer(plot_ui, layer);
                }
            }
        });
    }

    fn show_viewer_bis(&mut self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;

        // handle mouse input
        response.ctx.input(|i| {
            self.offset += response.drag_delta() / self.scale;

            if let Some(mut pos) = response.hover_pos() {
                self.offset += i.scroll_delta / self.scale;

                let old_scale = self.scale;
                self.scale *= i.zoom_delta();

                // zoom around mouse
                pos -= rect.min.to_vec2();
                let dz = 1. / old_scale - 1. / self.scale;
                self.offset -= pos.to_vec2() * dz;
            }
        });

        let to_screen =
            |p: Pos2| rect.min + (self.offset + Vec2::new(p.x, p.y)).to_vec2() * self.scale;

        // draw page size
        if let Some(page_size) = self.document.page_size {
            let page_size = Rect::from_points(&[
                to_screen(Pos2::ZERO),
                to_screen(Pos2::new(page_size.w as f32, page_size.h as f32)),
            ]);

            painter.rect_filled(
                page_size.clone().translate(Vec2::new(
                    SHADOW_OFFSET * self.scale,
                    SHADOW_OFFSET * self.scale,
                )),
                0.0,
                Color32::from_rgb(180, 180, 180),
            );

            painter.rect(
                page_size,
                0.0,
                Color32::WHITE,
                Stroke::new(1., Color32::from_rgb(128, 128, 128)),
            )
        }

        // draw layer data
        for (lid, layer) in &self.document.layers {
            if !self.layer_visibility.get(lid).unwrap_or(&true) {
                continue;
            }

            painter.extend(layer.paths.iter().map(
                |FlattenedPath {
                     data: path,
                     color,
                     stroke_width,
                 }| {
                    let pts = path
                        .iter()
                        .map(|pt| {
                            to_screen(Pos2 {
                                x: pt[0] as f32,
                                y: pt[1] as f32,
                            })
                        })
                        .collect::<Vec<Pos2>>();

                    if path.first() == path.last() {
                        Shape::closed_line(
                            pts,
                            Stroke::new(
                                *stroke_width as f32 * self.scale,
                                vsvg_to_egui_color(*color),
                            ),
                        )
                    } else {
                        Shape::line(
                            pts,
                            Stroke::new(
                                *stroke_width as f32 * self.scale,
                                vsvg_to_egui_color(*color),
                            ),
                        )
                    }
                },
            ))
        }
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
            ui.checkbox(&mut self.show_fat_lines, "Show fat lines");
            ui.add_enabled(
                self.show_fat_lines,
                egui::Checkbox::new(&mut self.show_fat_lines_debug, "Show fat lines debug lines"),
            );
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

    fn menu_debug(&mut self, ui: &mut Ui) {
        ui.menu_button("Debug", |ui| {
            ui.checkbox(&mut self.show_settings, "Show settings");
            ui.checkbox(&mut self.show_inspection, "Show inspection");
            ui.checkbox(&mut self.show_memory, "Show memory");
        });
    }
}

const SHADOW_OFFSET: f32 = 10.;

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
                self.menu_debug(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        let panel_frame = egui::Frame::central_panel(&ctx.style())
            .inner_margin(egui::style::Margin::same(0.))
            .fill(egui::Color32::from_rgb(242, 242, 242));
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| self.show_viewer_bis(ui));

        egui::Window::new("🔧 Settings")
            .open(&mut self.show_settings)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });

        egui::Window::new("🔍 Inspection")
            .open(&mut self.show_inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });

        egui::Window::new("📝 Memory")
            .open(&mut self.show_memory)
            .resizable(false)
            .show(ctx, |ui| {
                ctx.memory_ui(ui);
            });
    }
}

pub(crate) trait Show {
    fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>>;
}

impl Show for vsvg_core::Document {
    fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>> {
        let native_options = eframe::NativeOptions::default();
        let page_size = self.page_size;
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
                Box::new(Viewer::new(cc, polylines, control_points, page_size))
            }),
        )?;

        Ok(())
    }
}
