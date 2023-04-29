use crate::engine::{DisplayMode, DocumentData, Engine, ViewerOptions};
use eframe::egui_wgpu;
use egui::{Pos2, Rect, Sense, Ui};
use std::sync::{Arc, Mutex};

/// Widget to display a [`vsvg::Document`] in a egui application.
///
/// The widget is an egui wrapper around [`Engine`]. It holds the state needed for rendering, such
/// as the scale and pan offset.
///
/// It supports multiple UI features:
///  - GPU-accelerated rendering of the document, typically in the central panel
///  - helper UI functions to act on the widget state (e.g. viewing options and layer visibility)
#[derive(Debug, Default)]
pub struct DocumentWidget {
    /// polylines derived from the document
    document_data: Arc<DocumentData>,

    /// viewer options
    viewer_options: Arc<Mutex<ViewerOptions>>,

    /// pan offset
    ///
    /// The offset is expressed in SVG coordinates, not in pixels. `self.scale` can be used for
    /// conversion.
    offset: Pos2,

    /// scale factor
    scale: f32,

    /// should fit to view flag
    must_fit_to_view: bool,
}

impl DocumentWidget {
    pub fn new<'a>(
        cc: &'a eframe::CreationContext<'a>,
        document_data: Arc<DocumentData>,
    ) -> Option<Self> {
        let viewer_options = Arc::new(Mutex::new(ViewerOptions::default()));

        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        // prepare engine
        let engine = Engine::new(
            wgpu_render_state,
            document_data.clone(),
            viewer_options.clone(),
        );

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        let callback_resources = &mut wgpu_render_state.renderer.write().paint_callback_resources;
        callback_resources.insert(engine);

        Some(Self {
            document_data,
            viewer_options,
            offset: Pos2::ZERO,
            scale: 1.0,
            must_fit_to_view: true,
        })
    }

    pub fn ui(&mut self, ui: &mut Ui) {
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

    pub fn view_menu_ui(&mut self, ui: &mut Ui) {
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

    pub fn layer_menu_ui(&mut self, ui: &mut Ui) {
        ui.menu_button("Layer", |ui| {
            for (lid, layer) in &self.document_data.flattened_document.layers {
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

    fn fit_to_view(&mut self, viewport: &Rect) {
        let bounds = if let Some(page_size) = self.document_data.flattened_document.page_size {
            if page_size.w != 0.0 && page_size.h != 0.0 {
                Some(kurbo::Rect::from_points(
                    (0., 0.),
                    (page_size.w, page_size.h),
                ))
            } else {
                self.document_data.flattened_document.bounds()
            }
        } else {
            self.document_data.flattened_document.bounds()
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
}
