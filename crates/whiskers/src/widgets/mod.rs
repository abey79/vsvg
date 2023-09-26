pub mod bool;
pub mod numeric_widget;
pub mod point;
pub mod string_widget;

pub trait Widget<T> {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut T) -> egui::Response;
}

pub trait WidgetMapper<T> {
    type Type: Widget<T>;
}

#[macro_export]
macro_rules! register_widget_ui {
    ($t: ty, $ui: ty) => {
        impl $crate::widgets::WidgetMapper<$t> for $t {
            type Type = $ui;
        }
    };
}
