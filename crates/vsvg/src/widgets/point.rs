use crate::Point;
use egui::Ui;

/// Widget for the [`Point`] type.
#[derive(Default)]
pub struct PointWidget;

impl whiskers_widgets::Widget<Point> for PointWidget {
    fn ui(&self, ui: &mut Ui, label: &str, value: &mut Point) -> bool {
        ui.add(egui::Label::new(label));
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(value.x_mut()).speed(0.1))
                | ui.add(egui::DragValue::new(value.y_mut()).speed(0.1))
        })
        .inner
        .changed()
    }
}

whiskers_widgets::register_widget_ui!(Point, PointWidget);
