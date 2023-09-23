use egui::emath::Numeric;

#[derive(Default)]
pub struct NumericWidget<T: Numeric> {
    min: Option<T>,
    max: Option<T>,
    step: Option<T>,
    slider: bool,
}

impl<T: Numeric> NumericWidget<T> {
    pub fn min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    pub fn step(mut self, step: T) -> Self {
        self.step = Some(step);
        self
    }

    pub fn slider(mut self, slider: bool) -> Self {
        self.slider = slider;
        self
    }
}

impl<T: Numeric> super::Widget<T> for NumericWidget<T> {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut T) -> egui::Response {
        ui.horizontal(|ui| {
            ui.add(egui::Label::new(label));
            let range = self.min.unwrap_or(T::MIN)..=self.max.unwrap_or(T::MAX);
            if self.slider {
                let mut slider = egui::Slider::new(value, range);
                if let Some(step) = self.step {
                    slider = slider.step_by(step.to_f64());
                }
                ui.add(slider)
            } else {
                ui.add(egui::DragValue::new(value).clamp_range(range))
            }
        })
        .inner
    }
}
