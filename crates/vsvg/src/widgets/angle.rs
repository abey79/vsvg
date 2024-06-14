use crate::Angle;

/// A widget for [`crate::Angle`].
#[derive(Default)]
pub struct AngleWidget {
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    slider: bool,
    use_rad: bool,
}

impl AngleWidget {
    /// Sets the minimum value for the widget.
    #[must_use]
    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the maximum value for the widget.
    #[must_use]
    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the step value for the widget.
    ///
    /// This parameter is passed to [`egui::DragValue::speed`] in normal mode, and
    /// [`egui::Slider::step_by`] in slider mode.
    #[must_use]
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Sets whether the widget should be displayed as a slider or not.
    #[must_use]
    pub fn slider(mut self, slider: bool) -> Self {
        self.slider = slider;
        self
    }

    /// Use degrees as unit.
    #[must_use]
    pub fn deg(mut self, use_deg: bool) -> Self {
        self.use_rad = !use_deg;
        self
    }

    /// Use radians as unit.
    #[must_use]
    pub fn rad(mut self, use_rad: bool) -> Self {
        self.use_rad = use_rad;
        self
    }
}

impl whiskers_widgets::Widget<Angle> for AngleWidget {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut Angle) -> bool {
        ui.add(egui::Label::new(label));

        let mut angle_value = if self.use_rad {
            value.rad()
        } else {
            value.deg()
        };

        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width -= 40.0; // make some room for the combo box

            let range = self.min.unwrap_or(f64::MIN)..=self.max.unwrap_or(f64::MAX);
            let changed = if self.slider {
                let mut slider = egui::Slider::new(&mut angle_value, range);
                if let Some(step) = self.step {
                    slider = slider.step_by(step);
                }
                ui.add(slider).changed()
            } else {
                let mut drag_value = egui::DragValue::new(&mut angle_value).clamp_range(range);
                if let Some(step) = self.step {
                    drag_value = drag_value.speed(step);
                }
                ui.add(drag_value).changed()
            };

            ui.label(if self.use_rad { "rad" } else { "deg" });

            if changed {
                *value = if self.use_rad {
                    Angle::from_rad(angle_value)
                } else {
                    Angle::from_deg(angle_value)
                }
            }

            changed
        })
        .inner
    }
}

whiskers_widgets::register_widget_ui!(Angle, AngleWidget);
