use crate::engine::{DisplayMode, Engine, ViewerOptions};
use eframe::egui_wgpu;
use eframe::egui_wgpu::CallbackResources;
use eframe::epaint::PaintCallbackInfo;
use egui::{Pos2, Rect, Sense, Ui};
use std::sync::{Arc, Mutex};
use vsvg::{Document, DocumentTrait, LayerTrait};
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

/// Widget to display a [`Document`] in an egui application.
///
/// The widget is an egui wrapper around the internal `Engine` instance. It holds the state needed
/// for rendering, such as the scale and pan offset.
///
/// It supports multiple UI features:
///  - GPU-accelerated rendering of the document, typically in the central panel
///  - helper UI functions to act on the widget state (e.g. viewing options and layer visibility)
#[derive(Default)]
pub struct DocumentWidget {
    /// document to display
    document: Option<Arc<Document>>,

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
    /// Create a document widget.
    ///
    /// Initially, the document widget is empty. Use [`DocumentWidget::set_document()`] to set its
    /// content.
    #[must_use]
    pub(crate) fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let viewer_options = Arc::new(Mutex::new(ViewerOptions::default()));

        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        // prepare engine
        let engine = Engine::new(wgpu_render_state, viewer_options.clone());

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(engine);

        Some(Self {
            document: None,
            viewer_options,
            offset: Pos2::ZERO,
            scale: 1.0,
            must_fit_to_view: true,
        })
    }

    pub fn set_document(&mut self, doc: Arc<Document>) {
        self.document = Some(doc);
    }

    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.viewer_options
            .lock()
            .unwrap()
            .display_options
            .tolerance = tolerance;
    }

    #[must_use]
    pub fn vertex_count(&self) -> u64 {
        self.viewer_options.lock().unwrap().vertex_count
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn ui(&mut self, ui: &mut Ui) {
        vsvg::trace_function!();

        // do not actually allocate any space, so custom viewer code may use all of the central
        // panel
        let rect = ui.available_rect_before_wrap();
        let response = ui.interact(rect, ui.id(), Sense::click_and_drag());

        // fit to view on double click
        if response.double_clicked() {
            self.must_fit_to_view = true;
        }

        // fit to view on request
        if self.must_fit_to_view {
            self.fit_to_view(&rect);
        }

        // handle mouse input
        let old_offset = self.offset;
        let old_scale = self.scale;

        self.offset -= response.drag_delta() / self.scale;
        if let Some(mut pos) = response.hover_pos() {
            response.ctx.input(|i| {
                self.offset -= i.scroll_delta / self.scale;
                self.scale *= i.zoom_delta();
            });

            // zoom around mouse
            pos -= rect.min.to_vec2();
            let dz = 1. / old_scale - 1. / self.scale;
            self.offset += pos.to_vec2() * dz;
        }

        #[allow(clippy::float_cmp)]
        if old_offset != self.offset || old_scale != self.scale {
            self.must_fit_to_view = false;
        }

        // add the paint callback
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            DocumentWidgetCallback {
                document: self.document.clone(),
                origin: cgmath::Point2::new(self.offset.x, self.offset.y),
                scale: self.scale,
                rect,
            },
        ));
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
                &mut self
                    .viewer_options
                    .lock()
                    .unwrap()
                    .display_options
                    .show_display_vertices,
                "Show points",
            );
            ui.checkbox(
                &mut self
                    .viewer_options
                    .lock()
                    .unwrap()
                    .display_options
                    .show_pen_up,
                "Show pen-up trajectories",
            );
            ui.checkbox(
                &mut self
                    .viewer_options
                    .lock()
                    .unwrap()
                    .display_options
                    .show_bezier_handles,
                "Show control points",
            );
            ui.separator();
            if ui.button("Fit to view").clicked() {
                self.must_fit_to_view = true;
                ui.close_menu();
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("AA:");
                ui.add(egui::Slider::new(
                    &mut self.viewer_options.lock().unwrap().anti_alias,
                    0.0..=2.0,
                ))
                .on_hover_text("Renderer anti-aliasing (default: 0.5)");
            });

            ui.horizontal(|ui| {
                ui.label("Tol:");
                ui.add(
                    egui::Slider::new(
                        &mut self
                            .viewer_options
                            .lock()
                            .unwrap()
                            .display_options
                            .tolerance,
                        0.001..=10.0,
                    )
                    .logarithmic(true),
                )
                .on_hover_text("Tolerance for rendering curves (default: 0.01)");
            });
        });
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn layer_menu_ui(&mut self, ui: &mut Ui) {
        ui.menu_button("Layer", |ui| {
            let Some(document) = self.document.clone() else {
                return;
            };

            for (lid, layer) in &document.layers {
                let mut viewer_options = self.viewer_options.lock().unwrap();
                let visibility = viewer_options.layer_visibility.entry(*lid).or_insert(true);
                let mut label = format!("Layer {lid}");
                if let Some(name) = &layer.metadata().name {
                    label.push_str(&format!(": {name}"));
                }

                ui.checkbox(visibility, label);
            }
        });
    }

    fn fit_to_view(&mut self, viewport: &Rect) {
        vsvg::trace_function!();

        let Some(document) = self.document.clone() else {
            return;
        };

        let bounds = if let Some(page_size) = document.metadata().page_size {
            if page_size.w() != 0.0 && page_size.h() != 0.0 {
                Some(kurbo::Rect::from_points(
                    (0., 0.),
                    (page_size.w(), page_size.h()),
                ))
            } else {
                document.bounds()
            }
        } else {
            document.bounds()
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

struct DocumentWidgetCallback {
    document: Option<Arc<Document>>,
    origin: cgmath::Point2<f32>,
    scale: f32,
    rect: Rect,
}

impl egui_wgpu::CallbackTrait for DocumentWidgetCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        _egui_encoder: &mut CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        vsvg::trace_scope!("wgpu prepare callback");
        let engine: &mut Engine = callback_resources.get_mut().unwrap();

        if let Some(document) = self.document.clone() {
            engine.set_document(document);
        }

        engine.prepare(device, queue, self.rect, self.scale, self.origin);

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'a>,
        callback_resources: &'a CallbackResources,
    ) {
        vsvg::trace_scope!("wgpu paint callback");

        let engine: &Engine = callback_resources.get().unwrap();
        engine.paint(render_pass);
    }
}
