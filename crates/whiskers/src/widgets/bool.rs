/// Widget to display [`bool`] sketch parameters.
#[derive(Default)]
pub struct BoolWidget;

impl super::Widget<bool> for BoolWidget {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut bool) -> egui::Response {
        // empty first column
        ui.horizontal(|_| {});

        ui.checkbox(value, label.trim_end_matches(':'))
    }
}

crate::register_widget_ui!(bool, BoolWidget);
