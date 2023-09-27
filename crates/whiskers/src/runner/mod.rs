mod layout;
pub mod page_size;
#[cfg(not(target_arch = "wasm32"))]
mod save_ui_native;
#[cfg(target_arch = "wasm32")]
mod save_ui_wasm;
pub mod ui;

#[cfg(not(target_arch = "wasm32"))]
use save_ui_native::SaveUI;
#[cfg(target_arch = "wasm32")]
use save_ui_wasm::SaveUI;

use crate::Sketch;
use convert_case::Casing;
use eframe::Storage;
pub use layout::LayoutOptions;
pub use page_size::PageSizeOptions;
use rand::SeedableRng;
pub use ui::*;

use vsvg_viewer::DocumentWidget;

/// The [`Runner`] is the main entry point for executing a [`SketchApp`].
///
/// It can be configured using the builder pattern with the `with_*()` functions, and then run
/// using the [`run`] method.
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

    /// Options and UI for the sketch page size.
    page_size_options: PageSizeOptions,

    /// UI for the sketch layout options.
    layout_options: LayoutOptions,

    /// UI for saving the sketch.
    save_ui: SaveUI,

    // ========== seed stuff
    /// Controls whether the seed feature is enabled or not
    enable_seed: bool,

    /// Random seed used to generate the sketch.
    seed: u32,

    // ========== time stuff
    /// Controls whether the time feature is enabled or not
    enable_time: bool,

    /// Controls whether the time is running or not.
    playing: bool,

    /// Current sketch time.
    time: f64,

    /// Length of the time loop
    loop_time: f64,

    /// Is the time looping?
    is_looping: bool,

    /// Time of last loop.
    last_instant: Option<web_time::Instant>,

    _phantom: std::marker::PhantomData<&'a ()>,
}

// public methods
impl Runner<'_> {
    /// Create a new [`Runner`] with the provided [`SketchApp`] instance.
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
            save_ui,
            enable_seed: true,
            seed: 0,
            enable_time: true,
            playing: false,
            time: 0.0,
            loop_time: 3.0,
            is_looping: false,
            last_instant: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Sets the seed to a given value (default: 0).
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_seed_enabled(mut self, enabled: bool) -> Self {
        self.enable_seed = enabled;
        self
    }

    /// Randomizes the seed.
    pub fn with_random_seed(mut self) -> Self {
        self.seed = rand::random();
        self
    }

    /// Sets the default page size, which can be modified using the Page Size UI.
    pub fn with_page_size_options(self, page_size_options: impl Into<PageSizeOptions>) -> Self {
        Self {
            page_size_options: page_size_options.into(),
            ..self
        }
    }

    /// Sets the default layout options.
    pub fn with_layout_option(self, options: impl Into<LayoutOptions>) -> Self {
        Self {
            layout_options: options.into(),
            ..self
        }
    }

    /// Enables or disables the time feature.
    pub fn with_time_enabled(self, time: bool) -> Self {
        Self {
            enable_time: time,
            ..self
        }
    }

    /// Sets the initial time.
    pub fn with_time(self, time: f64) -> Self {
        Self { time, ..self }
    }

    /// Sets the initial play/pause state.
    pub fn with_time_playing(self, playing: bool) -> Self {
        Self { playing, ..self }
    }

    /// Sets the time loop length.
    pub fn with_loop_time(self, loop_time: f64) -> Self {
        Self { loop_time, ..self }
    }

    /// Enables or disables the time looping.
    pub fn with_looping_enabled(self, is_looping: bool) -> Self {
        Self { is_looping, ..self }
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

    fn time_ui(&mut self, ui: &mut egui::Ui) {
        collapsing_header(ui, "Animation", "", |ui| {
            ui.horizontal(|ui| {
                ui.label("time:");
                let max_time = if self.is_looping {
                    self.loop_time
                } else {
                    f64::MAX
                };
                ui.add_enabled(
                    !self.playing,
                    egui::DragValue::new(&mut self.time)
                        .speed(0.1)
                        .clamp_range(0.0..=max_time)
                        .suffix(" s"),
                )
                .dirty(self);
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.is_looping, "loop time:");
                ui.add_enabled(
                    self.is_looping,
                    egui::DragValue::new(&mut self.loop_time)
                        .speed(0.1)
                        .clamp_range(0.0..=f64::MAX)
                        .suffix(" s"),
                );
            });

            ui.horizontal(|ui| {
                if ui.button("reset").clicked() {
                    self.time = 0.0;
                    self.dirty();
                }
                if ui
                    .add_enabled(!self.playing, egui::Button::new("play"))
                    .clicked()
                {
                    self.playing = true;
                }
                if ui
                    .add_enabled(self.playing, egui::Button::new("pause"))
                    .clicked()
                {
                    self.playing = false;
                }
            });
        });
    }

    fn seed_ui(&mut self, ui: &mut egui::Ui) {
        collapsing_header(
            ui,
            "Random Number Generator",
            format!("seed: {}", self.seed),
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

    fn update_time(&mut self) {
        let now = web_time::Instant::now();

        if let Some(last_instant) = self.last_instant {
            if self.playing {
                let delta = now - last_instant;
                self.time += delta.as_secs_f64();

                if self.is_looping {
                    self.time %= self.loop_time;
                }

                self.dirty();
            }
        }

        self.last_instant = Some(web_time::Instant::now());
    }
}

impl vsvg_viewer::ViewerApp for Runner<'_> {
    fn update(
        &mut self,
        ctx: &egui::Context,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        if self.enable_time {
            self.update_time();
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

                            if self.enable_time {
                                self.time_ui(ui);
                            }

                            if self.enable_seed {
                                self.seed_ui(ui);
                            }

                            self.save_ui.ui(ui, self.last_sketch.as_ref());

                            collapsing_header(ui, "Sketch Parameters", "", |ui| {
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
                rng: rand_chacha::ChaCha8Rng::seed_from_u64(self.seed as u64),
                time: self.time,
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
    }

    fn save(&self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "whiskers-runner-save-ui", &self.save_ui);
        eframe::set_value(storage, "whiskers-layout-options", &self.layout_options);
        eframe::set_value(storage, "whiskers-page-size", &self.page_size_options);
    }
}
