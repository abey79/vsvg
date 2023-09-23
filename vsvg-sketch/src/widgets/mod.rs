pub mod numeric_widget;
pub mod string_widget;

pub trait Widget<T> {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut T);
}

pub trait WidgetMapper<T> {
    type Type: Widget<T>;
}

macro_rules! impl_numeric {
    ($t: ident) => {
        impl WidgetMapper<$t> for $t {
            type Type = numeric_widget::NumericWidget<$t>;
        }
    };
}

impl_numeric!(f32);
impl_numeric!(f64);
impl_numeric!(i8);
impl_numeric!(u8);
impl_numeric!(i16);
impl_numeric!(u16);
impl_numeric!(i32);
impl_numeric!(u32);
impl_numeric!(i64);
impl_numeric!(u64);
impl_numeric!(isize);
impl_numeric!(usize);

impl WidgetMapper<String> for String {
    type Type = string_widget::StringWidget;
}

#[macro_export]
macro_rules! register_widget_ui {
    ($t: ty, $ui: ty) => {
        impl ::vsvg_sketch::widgets::WidgetMapper<$t> for $t {
            type Type = $ui;
        }
    };
}
