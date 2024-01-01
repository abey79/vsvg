/// A widget for built-in numeric values.
///
/// This widget piggybacks on the [`Numeric`] trait for its implementation.
#[derive(Default)]
pub struct LengthWidget {
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    slider: bool,
    logarithmic: bool,
}

impl LengthWidget {
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

    /// Sets the widget to logarithmic mode. Implies [`slider(true)`].
    #[must_use]
    pub fn logarithmic(mut self, log: bool) -> Self {
        self.logarithmic = log;
        if log {
            self.slider = true;
        }
        self
    }
}

impl super::Widget<vsvg::Length> for LengthWidget {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut vsvg::Length) -> bool {
        ui.add(egui::Label::new(label));

        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width -= 40.0; // make some room for the combo box

            let range = self.min.unwrap_or(f64::MIN)..=self.max.unwrap_or(f64::MAX);
            let mut changed = if self.slider {
                let mut slider = egui::Slider::new(&mut value.value, range);
                if let Some(step) = self.step {
                    slider = slider.step_by(step);
                }
                if self.logarithmic {
                    slider = slider.logarithmic(true);
                }
                ui.add(slider).changed()
            } else {
                let mut drag_value = egui::DragValue::new(&mut value.value).clamp_range(range);
                if let Some(step) = self.step {
                    drag_value = drag_value.speed(step);
                }
                ui.add(drag_value).changed()
            };

            changed |=
                crate::widgets::unit_combo_box(ui, label, &mut value.unit, &vsvg::UNITS).changed();

            changed
        })
        .inner
    }
}

crate::register_widget_ui!(vsvg::Length, LengthWidget);
