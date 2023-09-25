mod save_ui;

use crate::Sketch;
use rand::SeedableRng;
use vsvg::{PageSize, Unit};

use vsvg_viewer::DocumentWidget;

/// The [`Runner`] is the main entry point for executing a [`SketchApp`].
///
/// It can be configured using the builder pattern with the `with_*()` functions, and then run
/// using the [`run`] method.
pub struct Runner<'a> {
    /// User-provided sketch app to run.
    app: Box<dyn crate::SketchApp>,

    /// Last produced sketch, for saving purposes.
    last_sketch: Option<Sketch>,

    /// Should the sketch be updated?
    dirty: bool,

    /// UI for saving the sketch.
    save_ui: save_ui::SaveUI,

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

    // ========== page size stuff
    /// The configured page size.
    page_size: PageSize,

    /// Enable the page size UI.
    page_size_locked: bool,

    _phantom: std::marker::PhantomData<&'a ()>,
}

// public methods
impl Runner<'_> {
    /// Create a new [`Runner`] with the provided [`SketchApp`] instance.
    pub fn new(app: impl crate::SketchApp + 'static) -> Self {
        Self {
            app: Box::new(app),
            last_sketch: None,
            dirty: true,
            save_ui: save_ui::SaveUI::default(),
            enable_seed: true,
            seed: 0,
            enable_time: true,
            playing: false,
            time: 0.0,
            loop_time: 3.0,
            is_looping: false,
            last_instant: None,
            page_size: PageSize::A4V,
            page_size_locked: false,
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
    pub fn with_page_size(self, page_size: PageSize) -> Self {
        Self {
            page_size,
            page_size_locked: false,
            ..self
        }
    }

    /// Sets the page size and disable the Page Size UI.
    pub fn with_locked_page_size(self, page_size: PageSize) -> Self {
        Self {
            page_size,
            page_size_locked: true,
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

impl Runner<'static> {
    /// Execute the sketch app.
    pub fn run(self) -> anyhow::Result<()> {
        vsvg_viewer::show_with_viewer_app(Box::new(self))
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

    fn page_size_ui(&mut self, ui: &mut egui::Ui) {
        ui.strong("Page Size");

        if self.page_size_locked {
            ui.label(format!("Locked to {}", self.page_size));
            return;
        }

        let mut new_page_size = self.page_size;

        ui.horizontal(|ui| {
            ui.label("format:");

            egui::ComboBox::from_id_source("sketch_page_size")
                .selected_text(new_page_size.to_format().unwrap_or("Custom"))
                .width(120.)
                .show_ui(ui, |ui| {
                    let orig = if matches!(new_page_size, PageSize::Custom(..)) {
                        new_page_size
                    } else {
                        PageSize::Custom(new_page_size.w(), new_page_size.h(), vsvg::Unit::PX)
                    };
                    ui.selectable_value(&mut new_page_size, orig, "Custom");

                    ui.separator();

                    for page_size in &vsvg::PAGE_SIZES {
                        ui.selectable_value(&mut new_page_size, *page_size, page_size.to_string());
                    }
                });

            if ui.button("flip").clicked() {
                new_page_size = new_page_size.flip();
            }
        });

        new_page_size = if let PageSize::Custom(mut w, mut h, mut unit) = new_page_size {
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut w)
                        .speed(1.0)
                        .clamp_range(0.0..=f64::MAX),
                );

                ui.label("x");
                ui.add(
                    egui::DragValue::new(&mut h)
                        .speed(1.0)
                        .clamp_range(0.0..=f64::MAX),
                );

                let orig_unit = unit;
                egui::ComboBox::from_id_source("sketch_page_size_unit")
                    .selected_text(unit.to_str())
                    .width(40.)
                    .show_ui(ui, |ui| {
                        const UNITS: [Unit; 8] = [
                            Unit::PX,
                            Unit::IN,
                            Unit::FT,
                            Unit::MM,
                            Unit::CM,
                            Unit::M,
                            Unit::PC,
                            Unit::PT,
                        ];

                        for u in &UNITS {
                            ui.selectable_value(&mut unit, *u, u.to_str());
                        }
                    });
                let factor = orig_unit.to_px() / unit.to_px();
                w *= factor;
                h *= factor;
            });

            PageSize::Custom(w, h, unit)
        } else {
            ui.label(format!(
                "{:.1}x{:.1} mm",
                new_page_size.w() / Unit::MM.to_px(),
                new_page_size.h() / Unit::MM.to_px()
            ));

            new_page_size
        };

        if new_page_size != self.page_size {
            self.page_size = new_page_size;
            self.dirty();
        }
    }

    fn time_ui(&mut self, ui: &mut egui::Ui) {
        ui.strong("Time");

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
    }

    fn seed_ui(&mut self, ui: &mut egui::Ui) {
        ui.strong("Random Number Generator");

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
                // let the UI breeze a little bit
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.spacing_mut().slider_width = 200.0;
                ui.visuals_mut().slider_trailing_fill = true;

                ui.vertical(|ui| {
                    self.page_size_ui(ui);
                    ui.separator();

                    if self.enable_time {
                        self.time_ui(ui);
                        ui.separator();
                    }

                    if self.enable_seed {
                        self.seed_ui(ui);
                        ui.separator();
                    }

                    self.save_ui.ui(ui, self.last_sketch.as_ref());
                    ui.separator();

                    ui.strong("Sketch Parameters");
                    let changed = egui::Grid::new("sketch_param_grid")
                        .num_columns(2)
                        .show(ui, |ui| self.app.ui(ui))
                        .inner;
                    self.set_dirty(changed);
                })
            });

        if self.dirty {
            self.dirty = false;

            ctx.request_repaint();

            let mut context = crate::context::Context {
                rng: rand_chacha::ChaCha8Rng::seed_from_u64(self.seed as u64),
                time: self.time,
            };

            let mut sketch = Sketch::new();
            sketch.page_size(self.page_size);
            self.app.update(&mut sketch, &mut context)?;
            document_widget.set_document_data(vsvg_viewer::DocumentData::new(sketch.document()));
            self.last_sketch = Some(sketch);

            // this removes the save result status, indicating that the sketch has changed since
            // last save
            self.save_ui.reset_error();
        }

        Ok(())
    }
}
