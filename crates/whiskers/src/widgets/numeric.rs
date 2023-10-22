use egui::emath::Numeric;

/// A widget for built-in numeric values.
///
/// This widget piggybacks on the [`Numeric`] trait for its implementation.
#[derive(Default)]
pub struct NumericWidget<T: Numeric> {
    min: Option<T>,
    max: Option<T>,
    step: Option<T>,
    slider: bool,
    logarithmic: bool,
}

impl<T: Numeric> NumericWidget<T> {
    /// Sets the minimum value for the widget.
    #[must_use]
    pub fn min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the maximum value for the widget.
    #[must_use]
    pub fn max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the step value for the widget.
    ///
    /// This parameter is passed to [`egui::DragValue::speed`] in normal mode, and
    /// [`egui::Slider::step_by`] in slider mode.
    #[must_use]
    pub fn step(mut self, step: T) -> Self {
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

impl<T: Numeric> super::Widget<T> for NumericWidget<T> {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut T) -> egui::Response {
        ui.add(egui::Label::new(label));
        let range = self.min.unwrap_or(T::MIN)..=self.max.unwrap_or(T::MAX);
        if self.slider {
            let mut slider = egui::Slider::new(value, range);
            if let Some(step) = self.step {
                slider = slider.step_by(step.to_f64());
            }
            if self.logarithmic {
                slider = slider.logarithmic(true);
            }
            ui.add(slider)
        } else {
            let mut drag_value = egui::DragValue::new(value).clamp_range(range);
            if let Some(step) = self.step {
                drag_value = drag_value.speed(step.to_f64());
            }
            ui.add(drag_value)
        }
    }
}

crate::register_widget_ui!(f32, NumericWidget<f32>);
crate::register_widget_ui!(f64, NumericWidget<f64>);
crate::register_widget_ui!(i8, NumericWidget<i8>);
crate::register_widget_ui!(u8, NumericWidget<u8>);
crate::register_widget_ui!(i16, NumericWidget<i16>);
crate::register_widget_ui!(u16, NumericWidget<u16>);
crate::register_widget_ui!(i32, NumericWidget<i32>);
crate::register_widget_ui!(u32, NumericWidget<u32>);
crate::register_widget_ui!(i64, NumericWidget<i64>);
crate::register_widget_ui!(u64, NumericWidget<u64>);
crate::register_widget_ui!(isize, NumericWidget<isize>);
crate::register_widget_ui!(usize, NumericWidget<usize>);
