use crate::engine::{DisplayMode, Engine, ViewerOptions};
use crate::frame_history::FrameHistory;
use eframe::{egui_wgpu, Frame};
use egui::{Color32, Pos2, Rect, Sense, Ui};
use vsvg_core::document::FlattenedDocument;

use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Viewer {
    /// polylines derived from the document
    #[serde(skip)]
    document: Arc<FlattenedDocument>,

    /// control points derived from the document
    #[serde(skip)]
    control_points: FlattenedDocument,

    /// viewer options
    viewer_options: Arc<Mutex<ViewerOptions>>,

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

impl Viewer {
    pub fn new<'a>(
        cc: &'a eframe::CreationContext<'a>,
        document: FlattenedDocument,
        control_points: FlattenedDocument,
    ) -> Option<Self> {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        /*
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }*/

        let document = Arc::new(document);
        let viewer_options = Arc::new(Mutex::new(ViewerOptions::default()));

        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        // prepare engine
        let engine = Engine::new(wgpu_render_state, document.clone(), viewer_options.clone());

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        let callback_resources = &mut wgpu_render_state.renderer.write().paint_callback_resources;
        callback_resources.insert(engine);

        Some(Viewer {
            document,
            control_points,
            viewer_options,
            offset: Pos2::ZERO,
            scale: 1.0,
            must_fit_to_view: true,
            show_settings: false,
            show_inspection: false,
            show_memory: false,
            frame_history: FrameHistory::default(),
        })
    }

    #[allow(clippy::unused_self)]
    fn menu_file(&self, frame: &mut Frame, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                frame.close();
            }
        });
    }

    fn menu_view(&mut self, ui: &mut Ui) {
        ui.menu_button("View", |ui| {
            ui.menu_button("Display Mode", |ui| {
                if ui
                    .radio_value(
                        &mut self.viewer_options.lock().unwrap().display_mode,
                        DisplayMode::Preview,
                        "Preview",
                    )
                    .clicked()
                {
                    ui.close_menu();
                };
                if ui
                    .radio_value(
                        &mut self.viewer_options.lock().unwrap().display_mode,
                        DisplayMode::Outline,
                        "Outline",
                    )
                    .clicked()
                {
                    ui.close_menu();
                };
            });
            ui.separator();
            ui.checkbox(
                &mut self.viewer_options.lock().unwrap().show_point,
                "Show points",
            );
            ui.checkbox(
                &mut self.viewer_options.lock().unwrap().show_pen_up,
                "Show pen-up trajectories",
            );
            ui.checkbox(
                &mut self.viewer_options.lock().unwrap().show_control_points,
                "Show control points",
            );
            ui.separator();
            if ui.button("Fit to view").clicked() {
                self.must_fit_to_view = true;
                ui.close_menu();
            }
        });
    }

    fn menu_layer(&mut self, ui: &mut Ui) {
        ui.menu_button("Layer", |ui| {
            for (lid, layer) in &self.document.layers {
                let mut viewer_options = self.viewer_options.lock().unwrap();
                let visibility = viewer_options.layer_visibility.entry(*lid).or_insert(true);
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
            if ui.button("Show settings window").clicked() {
                self.show_settings = true;
                ui.close_menu();
            }
            if ui.button("Show inspection window").clicked() {
                self.show_inspection = true;
                ui.close_menu();
            }
            if ui.button("Show memory window").clicked() {
                self.show_memory = true;
                ui.close_menu();
            }
        });
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

        #[allow(clippy::cast_possible_truncation)]
        {
            let (w, h) = (bounds.width() as f32, bounds.height() as f32);
            let (view_w, view_h) = (viewport.width(), viewport.height());

            self.scale = 0.95 * f32::min(view_w / w, view_h / h);

            self.offset = Pos2::new(
                bounds.x0 as f32 - (view_w / self.scale - w) / 2.0,
                bounds.y0 as f32 - (view_h / self.scale - h) / 2.0,
            );
        }
    }

    fn paint_viewer(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

        // fit to view on request
        if self.must_fit_to_view {
            self.fit_to_view(&rect);
        }

        // handle mouse input
        let old_offset = self.offset;
        let old_scale = self.scale;
        response.ctx.input(|i| {
            self.offset -= response.drag_delta() / self.scale;

            if let Some(mut pos) = response.hover_pos() {
                self.offset -= i.scroll_delta / self.scale;

                let old_scale = self.scale;
                self.scale *= i.zoom_delta();

                // zoom around mouse
                pos -= rect.min.to_vec2();
                let dz = 1. / old_scale - 1. / self.scale;
                self.offset += pos.to_vec2() * dz;
            }
        });

        #[allow(clippy::float_cmp)]
        if old_offset != self.offset || old_scale != self.scale {
            self.must_fit_to_view = false;
        }

        // The callback function for WGPU is in two stages: prepare, and paint.
        //
        // The prepare callback is called every frame before paint and is given access to the wgpu
        // Device and Queue, which can be used, for instance, to update buffers and uniforms before
        // rendering.
        //
        // You can use the main `CommandEncoder` that is passed-in, return an arbitrary number
        // of user-defined `CommandBuffer`s, or both.
        // The main command buffer, as well as all user-defined ones, will be submitted together
        // to the GPU in a single call.
        //
        // The paint callback is called after prepare and is given access to the render pass, which
        // can be used to issue draw commands.

        let scale = self.scale;
        let origin = cgmath::Point2::new(self.offset.x, self.offset.y);

        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _encoder, paint_callback_resources| {
                let engine: &mut Engine = paint_callback_resources.get_mut().unwrap();
                engine.prepare(device, queue, rect, scale, origin);
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let engine: &Engine = paint_callback_resources.get().unwrap();
                engine.paint(render_pass);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}

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
            .show(ctx, |ui| self.paint_viewer(ui, frame));

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
