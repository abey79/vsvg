use eframe::Frame;

use crate::frame_history::FrameHistory;
use egui::epaint::Vertex;
use egui::{Color32, Mesh, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::error::Error;
use vsvg_core::flattened_layer::FlattenedLayer;
use vsvg_core::FlattenedPath;
use vsvg_core::{document::FlattenedDocument, LayerID};
use vsvg_viewer::triangulation::{build_fat_line, FatLineBuffer};

pub struct MeshBuffer<F: Fn(Pos2) -> Pos2> {
    mesh: Mesh,
    to_screen: F,
}

impl<F: Fn(Pos2) -> Pos2> MeshBuffer<F> {
    pub fn new(to_screen: F) -> Self {
        Self {
            mesh: Mesh::default(),
            to_screen,
        }
    }
}

impl<F: Fn(Pos2) -> Pos2> FatLineBuffer for MeshBuffer<F> {
    fn push_vertex(&mut self, p: kurbo::Point) -> usize {
        let len = self.mesh.vertices.len();
        let mut rng = ChaCha8Rng::seed_from_u64(len as u64);

        self.mesh.vertices.push(Vertex {
            pos: (self.to_screen)(Pos2::new(p.x as f32, p.y as f32)),
            uv: Pos2::ZERO,
            color: Color32::from_rgba_unmultiplied(rng.gen(), rng.gen(), rng.gen(), 70),
        });
        len
    }

    fn push_triangle(&mut self, i1: usize, i2: usize, i3: usize) {
        self.mesh.indices.extend(&[i1 as u32, i2 as u32, i3 as u32]);
    }

    fn set_vertex(&mut self, i: usize, p: kurbo::Point) {
        self.mesh.vertices[i].pos = Pos2::new(p.x as f32, p.y as f32);
    }

    fn get_vertex(&self, i: usize) -> kurbo::Point {
        let p = self.mesh.vertices[i].pos;
        kurbo::Point::new(p.x as f64, p.y as f64)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub(crate) struct Viewer {
    /// polylines derived from the document
    #[serde(skip)]
    document: FlattenedDocument,

    /// control points derived from the document
    #[serde(skip)]
    control_points: FlattenedDocument,

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

    /// should fit to view flag
    #[serde(skip)]
    must_fit_to_view: bool,

    /// Show settings window.
    show_settings: bool,

    /// Show inspection window.
    show_inspection: bool,

    /// Show memory window.
    show_memory: bool,

    /// Record frame performance
    #[serde(skip)]
    frame_history: FrameHistory,
}

fn vsvg_to_egui_color(val: vsvg_core::Color) -> Color32 {
    Color32::from_rgba_unmultiplied(val.r, val.g, val.b, val.a)
}

impl Viewer {
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        document: FlattenedDocument,
        control_points: FlattenedDocument,
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
            show_point: false,
            show_grid: false,
            show_control_points: false,
            show_fat_lines: false,
            show_fat_lines_debug: false,
            layer_visibility: HashMap::new(),
            offset: Pos2::ZERO,
            scale: 1.0,
            must_fit_to_view: true,
            show_settings: false,
            show_inspection: false,
            show_memory: false,
            frame_history: FrameHistory::default(),
        }
    }

    fn fit_to_view(&mut self, viewport: &Rect) {
        let bounds = if let Some(page_size) = self.document.page_size {
            if page_size.w != 0.0 && page_size.h != 0.0 {
                Some(kurbo::Rect::from_points(
                    (0., 0.),
                    (page_size.w, page_size.h),
                ))
            } else {
                self.document.bounds()
            }
        } else {
            self.document.bounds()
        };

        if bounds.is_none() {
            return;
        }
        let bounds = bounds.expect("bounds is not none");

        let (w, h) = (bounds.width() as f32, bounds.height() as f32);
        let (view_w, view_h) = (viewport.width(), viewport.height());

        self.scale = 0.95 * f32::min(view_w / w, view_h / h);
        self.offset = Pos2::new(
            bounds.x0 as f32 + (view_w / self.scale - w) / 2.0,
            bounds.y0 as f32 + (view_h / self.scale - h) / 2.0,
        );
    }

    fn show_viewer(&mut self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;

        // fit to view on request
        if self.must_fit_to_view {
            self.fit_to_view(&rect);
        }

        // handle mouse input
        let old_offset = self.offset;
        let old_scale = self.scale;
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

        if old_offset != self.offset || old_scale != self.scale {
            self.must_fit_to_view = false;
        }

        let (off, sc) = (self.offset, self.scale);
        let to_screen = |p: Pos2| rect.min + (off + Vec2::new(p.x, p.y)).to_vec2() * sc;

        // draw page size
        self.paint_page_size(&painter, to_screen);

        // draw layer data

        for (lid, layer) in &self.document.layers {
            if !self.layer_visibility.get(lid).unwrap_or(&true) {
                continue;
            }

            if self.show_fat_lines {
                self.paint_layer_fat_lines(&painter, to_screen, layer);
            } else {
                self.paint_layer(&painter, to_screen, layer);
            }
        }
    }

    fn paint_layer<F: Fn(Pos2) -> Pos2>(
        &self,
        painter: &Painter,
        to_screen: F,
        layer: &FlattenedLayer,
    ) {
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
                            x: pt.x() as f32,
                            y: pt.y() as f32,
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

    fn paint_layer_fat_lines<F: Fn(Pos2) -> Pos2>(
        &self,
        painter: &Painter,
        to_screen: F,
        layer: &FlattenedLayer,
    ) {
        let mut buffer = MeshBuffer::new(&to_screen);
        for FlattenedPath {
            data: path,
            color: _color,
            stroke_width,
        } in &layer.paths
        {
            build_fat_line(path, *stroke_width, &mut buffer);
        }

        painter.add(buffer.mesh);

        // plot debug stuff
        if self.show_fat_lines_debug {
            painter.extend(layer.paths.iter().map(|FlattenedPath { data: path, .. }| {
                Shape::line(
                    path.iter()
                        .map(|pt| {
                            to_screen(Pos2 {
                                x: pt.x() as f32,
                                y: pt.y() as f32,
                            })
                        })
                        .collect::<Vec<Pos2>>(),
                    Stroke::new(1.0, Color32::LIGHT_GREEN),
                )
            }))
        }
    }

    fn paint_page_size<F: Fn(Pos2) -> Pos2>(&self, painter: &Painter, to_screen: F) {
        if let Some(page_size) = self.document.page_size {
            let page_size = Rect::from_points(&[
                to_screen(Pos2::ZERO),
                to_screen(Pos2::new(page_size.w as f32, page_size.h as f32)),
            ]);

            painter.rect_filled(
                page_size.translate(Vec2::new(
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
            ui.separator();
            if ui.button("Fit to view").clicked() {
                self.must_fit_to_view = true;
            }
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
                self.frame_history.ui(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        let panel_frame = egui::Frame::central_panel(&ctx.style())
            .inner_margin(egui::style::Margin::same(0.))
            .fill(Color32::from_rgb(242, 242, 242));
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| self.show_viewer(ui));

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

        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
    }
}

pub(crate) trait Show {
    fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>>;
}

impl Show for vsvg_core::Document {
    fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>> {
        let native_options = eframe::NativeOptions::default();
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
                Box::new(Viewer::new(cc, polylines, control_points))
            }),
        )?;

        Ok(())
    }
}
