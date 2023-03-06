use eframe::Frame;
use egui::plot::PlotUi;
use egui::Ui;
use std::collections::HashMap;
use std::error::Error;
use vsvg_core::flattened_layer::FlattenedLayer;
use vsvg_core::{document::FlattenedDocument, LayerID, PageSize, Polyline};
use vsvg_core::{FlattenedPath, Transforms};

type Triangle = (usize, usize, usize);

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

    /// This function computes a triangulation to render fat lines.
    fn build_fat_line(
        line: &Polyline,
        pen_width: f64,
        vertices: &mut Vec<kurbo::Point>,
        triangles: &mut Vec<Triangle>,
    ) {
        //todo: handle closing lines

        let len = line.len();

        if len < 2 {
            //todo: handle len == 1 => two triangle for a single square
            return;
        }

        let mut push_v = |p| {
            vertices.push(p);
            vertices.len() - 1
        };

        let mut push_t = |i1, i2, i3| {
            triangles.push((i1, i2, i3));
        };

        // The strategy to handle closing lines is the following:
        // - generate the first two vertices as normal
        // - append line[1] at the end of the line iterator, so a full extra segment is
        //   generated (remember: line[0] is already the same as line[len - 1])
        // - instead of generating the last two vertices, merge the first two vertices
        let closing = len > 3 && line[0] == line[len - 1];

        let w = pen_width / 2.0;

        let mut p1 = kurbo::Point::new(line[0][0], line[0][1]);
        let mut p2 = kurbo::Point::new(line[1][0], line[1][1]);

        let mut v1 = (p2 - p1).normalize();
        let mut n1 = kurbo::Vec2 { x: -v1.y, y: v1.x };
        let mut critical_length_1 = (p2 - p1 + w * n1).hypot();

        // note: idx1 is always chosen to be on the side of the normal
        let mut idx1 = push_v(p1 + w * (-v1 + n1));
        let mut idx2 = push_v(p1 + w * (-v1 - n1));

        // remember those to close the loop
        let first_idx1 = idx1;
        let first_idx2 = idx2;

        let mut v0: kurbo::Vec2;
        let mut n0: kurbo::Vec2;
        let mut critical_length_0: f64;

        // if `closing`, the iterator has length len-1
        let iter = line[2..].iter().chain(if closing {
            line[1..2].iter()
        } else {
            [].iter()
        });
        let mut post_process_close = true;
        for (i, new_pt) in iter.enumerate() {
            // this is when we must "seam" the triangulation back to the first two vertices
            let finish_close = closing && i == len - 2;

            // p0 is where we're departing from, but not actually needed
            p1 = p2;
            p2 = kurbo::Point::new(new_pt[0], new_pt[1]);

            v0 = v1;
            n0 = n1;
            v1 = (p2 - p1).normalize();
            n1 = kurbo::Vec2 { x: -v1.y, y: v1.x };

            let v0v1 = kurbo::Vec2::dot(v0, v1);
            let d = kurbo::Vec2::cross(v0, v1).signum();
            let miter = (n0 + n1).normalize();
            let half_join = w / miter.dot(n0);

            critical_length_0 = critical_length_1;
            critical_length_1 = (p2 - p1 + w * n1).hypot();
            let restart = half_join >= critical_length_0 || half_join >= critical_length_1;

            if restart {
                // We interrupt the line here and restart a new one. This means that we must emit
                // two vertices at p1 and aligned with p0, then the two related triangles. Then we
                // must create two other vertices at p1, aligned with p2, ready for the next point.

                // In case we're closing and we must over-draw, we must emit two new closing
                // vertices, and related triangles, but skip creating new vertices for the next
                // point.

                let idx3 = push_v(p1 + w * (v0 + n0));
                let idx4 = push_v(p1 + w * (v0 - n0));
                push_t(idx1, idx2, idx3);
                push_t(idx2, idx3, idx4);

                if !finish_close {
                    // prepare for next line
                    idx1 = push_v(p1 + w * (-v1 + n1));
                    idx2 = push_v(p1 + w * (-v1 - n1));
                } else {
                    post_process_close = false;
                }
            } else {
                let idx3: usize;
                let idx4: usize;

                if v0v1 >= 0. {
                    // corner is less than 90° => no miter triangle is needed
                    idx3 = push_v(p1 + half_join * miter);
                    idx4 = push_v(p1 - half_join * miter);

                    push_t(idx1, idx2, idx3);
                    push_t(idx2, idx3, idx4);
                } else {
                    // corner is more than 90° => miter triangle is needed
                    // TBD: should the limit *really* be at 90°? Triangle count could be limited by
                    // setting the threshold a bit higher...

                    let idx5: usize;

                    if d == 1. {
                        idx3 = push_v(p1 + half_join * miter);
                        idx4 = push_v(p1 + w * (-v1 - n1));
                        idx5 = push_v(p1 + w * (v0 - n0));
                        push_t(idx1, idx2, idx3);
                        push_t(idx2, idx3, idx5);
                    } else {
                        idx3 = push_v(p1 + w * (-v1 + n1));
                        idx4 = push_v(p1 - half_join * miter);
                        idx5 = push_v(p1 + w * (v0 + n0));
                        push_t(idx1, idx2, idx5);
                        push_t(idx2, idx4, idx5);
                    }
                    push_t(idx3, idx4, idx5);
                }

                idx1 = idx3;
                idx2 = idx4;
            }
        }

        if closing {
            if post_process_close {
                // Ideally, those last two vertices could be avoided by reusing the first two. I'm
                // not sure the additional CPU cycles are worth the memory savings...
                vertices[first_idx1] = vertices[idx1];
                vertices[first_idx2] = vertices[idx2];
            }
        } else {
            // finish off the line
            let idx3 = push_v(p2 + w * (v1 + n1));
            let idx4 = push_v(p2 + w * (v1 - n1));
            push_t(idx1, idx2, idx3);
            push_t(idx2, idx3, idx4);
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
            Viewer::build_fat_line(path, *stroke_width, &mut vertices, &mut triangles);
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
