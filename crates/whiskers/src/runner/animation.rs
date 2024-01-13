use whiskers_widgets::collapsing_header;

/// Controls the animation feature of the runner.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnimationOptions {
    /// Controls whether the time is running or not.
    playing: bool,

    /// Current sketch time.
    #[serde(skip)]
    pub(crate) time: f64,

    /// Length of the time loop
    pub(crate) loop_time: f64,

    /// Is the time looping?
    is_looping: bool,

    /// Time of last loop.
    #[serde(skip)]
    last_instant: Option<web_time::Instant>,
}

impl Default for AnimationOptions {
    fn default() -> Self {
        Self {
            playing: false,
            time: 0.0,
            loop_time: 10.0,
            is_looping: false,
            last_instant: None,
        }
    }
}

impl AnimationOptions {
    /// Constructor function for a looping [`AnimationOptions`] with the provided loop time.
    pub fn looping(loop_time: f64) -> Self {
        Self {
            loop_time,
            is_looping: true,
            ..Default::default()
        }
    }

    /// Sets the [`AnimationOptions`] to play mode by default.
    #[must_use]
    pub fn play(self) -> Self {
        Self {
            playing: true,
            ..self
        }
    }
}

impl AnimationOptions {
    #[must_use]
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        collapsing_header(ui, "Animation", "", false, |ui| {
            ui.horizontal(|ui| {
                ui.label("time:");
                let max_time = if self.is_looping {
                    self.loop_time
                } else {
                    f64::MAX
                };
                changed |= ui
                    .add_enabled(
                        !self.playing,
                        egui::DragValue::new(&mut self.time)
                            .speed(0.1)
                            .clamp_range(0.0..=max_time)
                            .suffix(" s"),
                    )
                    .changed();
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
                    changed = true;
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

        changed
    }

    #[must_use]
    pub(crate) fn update_time(&mut self) -> bool {
        let now = web_time::Instant::now();

        let mut changed = false;

        if let Some(last_instant) = self.last_instant {
            if self.playing {
                let delta = now - last_instant;
                self.time += delta.as_secs_f64();

                if self.is_looping {
                    self.time %= self.loop_time;
                }

                changed = true;
            }
        }

        self.last_instant = Some(web_time::Instant::now());

        changed
    }
}
