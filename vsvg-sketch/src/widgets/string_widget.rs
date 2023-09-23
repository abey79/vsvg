#[derive(Default)]
pub struct StringWidget;

impl super::Widget<String> for StringWidget {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut String) {
        ui.horizontal(|ui| {
            ui.add(egui::Label::new(label));
            ui.add(egui::TextEdit::singleline(value));
        });
    }
}
