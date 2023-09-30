mod animation;
mod layout;
mod page_size;
#[cfg(not(target_arch = "wasm32"))]
mod save_ui_native;
#[cfg(target_arch = "wasm32")]
mod save_ui_wasm;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
use save_ui_native::SaveUI;
#[cfg(target_arch = "wasm32")]
use save_ui_wasm::SaveUI;

use crate::Sketch;
pub use animation::AnimationOptions;
use convert_case::Casing;
use eframe::Storage;
pub use layout::LayoutOptions;
pub use page_size::PageSizeOptions;
use rand::SeedableRng;
pub use ui::*;

use vsvg_viewer::DocumentWidget;

/// The [`Runner`] is the main entry point for executing a [`crate::SketchApp`].
///
/// It can be configured using the builder pattern with the `with_*()` functions, and then run
/// using the [`Runner::run`] method.
///
/// [`Runner`] implements [`vsvg_viewer::ViewerApp`] to actually display the sketch with a custom,
/// interactive UI.
pub struct Runner<'a> {
    /// User-provided sketch app to run.
    app: Box<dyn crate::SketchApp>,

    /// Last produced sketch, for saving purposes.
    last_sketch: Option<Sketch>,

    /// Should the sketch be updated?
    dirty: bool,

    /// Options and UI for the page size panel.
    page_size_options: PageSizeOptions,

    /// Options and UI for the layout panel.
    layout_options: LayoutOptions,

    /// Options and UI for the animation panel.
    animation_options: AnimationOptions,

    /// Options and UI for save panel.
    save_ui: SaveUI,

    // ========== seed stuff
    /// Random seed used to generate the sketch.
    seed: u32,

    // ========== time stuff
    _phantom: std::marker::PhantomData<&'a ()>,
}

// public methods
impl Runner<'_> {
    /// Create a new [`Runner`] with the provided [`crate::SketchApp`] instance.
    pub fn new(app: impl crate::SketchApp + 'static) -> Self {
        let mut save_ui = SaveUI::default();
        #[allow(clippy::field_reassign_with_default)]
        {
            save_ui.base_name = app.name().to_case(convert_case::Case::Snake);
        }

        Self {
            app: Box::new(app),
            last_sketch: None,
            dirty: true,
            page_size_options: PageSizeOptions::default(),
            layout_options: LayoutOptions::default(),
            animation_options: AnimationOptions::default(),
            save_ui,
            seed: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Sets the seed to a given value (default: 0).
    #[must_use]
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    /// Randomizes the seed.
    #[must_use]
    pub fn with_random_seed(mut self) -> Self {
        self.seed = rand::random();
        self
    }

    /// Sets the default page size, which can be modified using the Page Size UI.
    #[must_use]
    pub fn with_page_size_options(self, page_size_options: impl Into<PageSizeOptions>) -> Self {
        Self {
            page_size_options: page_size_options.into(),
            ..self
        }
    }

    /// Sets the default layout options.
    #[must_use]
    pub fn with_layout_option(self, options: impl Into<LayoutOptions>) -> Self {
        Self {
            layout_options: options.into(),
            ..self
        }
    }

    /// Sets the default animation options.
    #[must_use]
    pub fn with_animation_options(self, options: impl Into<AnimationOptions>) -> Self {
        Self {
            animation_options: options.into(),
            ..self
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Runner<'static> {
    /// Execute the sketch app.
    pub fn run(self) -> anyhow::Result<()> {
        vsvg_viewer::show_with_viewer_app(self)
    }
}

/// Convenience trait to be used with [`egui::Response`] for setting the [`Runner`] dirty
/// flag.
trait DirtySetter {
    fn dirty(&self, runner: &mut Runner);
}

impl DirtySetter for egui::Response {
    fn dirty(&self, runner: &mut Runner) {
        runner.set_dirty(self.changed());
    }
}

// private methods
impl Runner<'_> {
    /// Set the dirty flag.
    #[inline]
    fn dirty(&mut self) {
        self.dirty = true;
    }

    /// Conditionally set the dirty flag.
    ///
    /// Passing `false` does not clear the dirty flag if it was already set.
    #[inline]
    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty || self.dirty;
    }

    fn seed_ui(&mut self, ui: &mut egui::Ui) {
        collapsing_header(
            ui,
            "Random Number Generator",
            format!("seed: {}", self.seed),
            false,
            |ui| {
                ui.horizontal(|ui| {
                    ui.label("seed:");
                    ui.add(egui::DragValue::new(&mut self.seed).speed(1.0))
                        .dirty(self);
                });

                ui.horizontal(|ui| {
                    if ui.button("random").clicked() {
                        self.seed = rand::random();
                        self.dirty();
                    }
                    if ui
                        .add_enabled(self.seed != 0, egui::Button::new("prev"))
                        .clicked()
                    {
                        self.seed = self.seed.saturating_sub(1);
                        self.dirty();
                    }
                    if ui
                        .add_enabled(self.seed != u32::MAX, egui::Button::new("next"))
                        .clicked()
                    {
                        self.seed = self.seed.saturating_add(1);
                        self.dirty();
                    }
                });
            },
        );
    }
}

impl vsvg_viewer::ViewerApp for Runner<'_> {
    fn update(
        &mut self,
        ctx: &egui::Context,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        if self.animation_options.update_time() {
            self.dirty();
        }

        egui::SidePanel::right("right_panel")
            .default_width(200.)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.spacing_mut().slider_width = 170.0;
                ui.visuals_mut().slider_trailing_fill = true;
                ui.visuals_mut().collapsing_header_frame = true;

                egui::ScrollArea::vertical()
                    .id_source("side_bar_scroll")
                    .show(ui, |ui| {
                        // let the UI breeze a little bit

                        ui.vertical(|ui| {
                            if self.page_size_options.ui(ui) {
                                self.dirty();
                            }

                            if self.layout_options.ui(ui) {
                                self.dirty();
                            }

                            if self.animation_options.ui(ui) {
                                self.dirty();
                            }

                            self.seed_ui(ui);

                            self.save_ui.ui(ui, self.last_sketch.as_ref());

                            collapsing_header(ui, "Sketch Parameters", "", true, |ui| {
                                let changed = egui::Grid::new("sketch_param_grid")
                                    .num_columns(2)
                                    .show(ui, |ui| self.app.ui(ui))
                                    .inner;
                                self.set_dirty(changed);
                            });
                        })
                    });
            });

        if self.dirty {
            self.dirty = false;

            ctx.request_repaint();

            let mut context = crate::context::Context {
                rng: rand_chacha::ChaCha8Rng::seed_from_u64(u64::from(self.seed)),
                time: self.animation_options.time,
                loop_time: self.animation_options.loop_time,
            };

            let mut sketch = Sketch::new();
            sketch.page_size(self.page_size_options.page_size);
            self.app.update(&mut sketch, &mut context)?;
            self.layout_options.apply(&mut sketch);
            document_widget.set_document_data(vsvg_viewer::DocumentData::new(sketch.document()));
            self.last_sketch = Some(sketch);

            // this removes the save result status, indicating that the sketch has changed since
            // last save
            #[cfg(not(target_arch = "wasm32"))]
            self.save_ui.reset_error();
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn options(&self, native_option: &mut eframe::NativeOptions) {
        native_option.app_id = Some(format!("vsvg.sketch.{}", self.title()));
    }

    fn title(&self) -> String {
        self.app.name()
    }

    fn load(&mut self, storage: &dyn Storage) {
        let save_ui: Option<SaveUI> = eframe::get_value(storage, "whiskers-runner-save-ui");
        #[allow(unused_mut)]
        if let Some(mut save_ui) = save_ui {
            #[cfg(not(target_arch = "wasm32"))]
            save_ui.update_dest_dir();

            self.save_ui = save_ui;
        }

        if let Some(layout_options) = eframe::get_value(storage, "whiskers-layout-options") {
            self.layout_options = layout_options;
        }

        if !self.page_size_options.locked {
            if let Some(page_size_options) = eframe::get_value(storage, "whiskers-page-size") {
                self.page_size_options = page_size_options;
            }
        }

        if let Some(animation_options) = eframe::get_value(storage, "whiskers-animation") {
            self.animation_options = animation_options;
        }
    }

    fn save(&self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "whiskers-runner-save-ui", &self.save_ui);
        eframe::set_value(storage, "whiskers-layout-options", &self.layout_options);
        eframe::set_value(storage, "whiskers-page-size", &self.page_size_options);
        eframe::set_value(storage, "whiskers-animation", &self.animation_options);
    }
}
