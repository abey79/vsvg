use crate::Sketch;
use rand::SeedableRng;

use vsvg_viewer::DocumentWidget;

pub(crate) struct SketchRunner {
    /// User-provided sketch app to run.
    pub(crate) app: Box<dyn crate::SketchApp>,

    /// Should the sketch be updated?
    dirty: bool,

    /// Random seed used to generate the sketch.
    seed: u32,

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
}

/// Convenience trait to be used with [`egui::Response`] for setting the [`SketchRunner`] dirty
/// flag.
trait DirtySetter {
    fn dirty(&self, runner: &mut SketchRunner);
}

impl DirtySetter for egui::Response {
    fn dirty(&self, runner: &mut SketchRunner) {
        runner.set_dirty(self.changed());
    }
}

impl SketchRunner {
    pub(crate) fn new(app: Box<dyn crate::SketchApp>) -> Self {
        Self {
            app,
            dirty: true,
            seed: 0,
            playing: false,
            time: 0.0,
            loop_time: 3.0,
            is_looping: false,
            last_instant: None,
        }
    }

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

impl vsvg_viewer::ViewerApp for SketchRunner {
    fn update(
        &mut self,
        ctx: &egui::Context,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        let mut sketch = Sketch::new();

        self.update_time();

        egui::SidePanel::right("right_panel")
            .default_width(200.)
            .show(ctx, |ui| {
                // let the UI breeze a little bit
                ui.spacing_mut().item_spacing.y = 6.0;

                ui.vertical(|ui| {
                    self.time_ui(ui);
                    ui.separator();

                    self.seed_ui(ui);
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

            self.app.update(&mut sketch, &mut context)?;
            document_widget.set_document_data(vsvg_viewer::DocumentData::new(sketch.document()));
        }

        Ok(())
    }
}
