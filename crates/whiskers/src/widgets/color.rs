/// Widget to display [`vsvg::Color`] sketch parameters.
#[derive(Default)]
pub struct ColorWidget;

impl super::Widget<vsvg::Color> for ColorWidget {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut vsvg::Color) -> egui::Response {
        // empty first column
        ui.label(label);

        let mut color_components: [f32; 4] = (*value).into();
        let resp = ui.color_edit_button_rgba_unmultiplied(&mut color_components);
        if resp.changed() {
            *value = color_components.into();
        }

        resp
    }
}

crate::register_widget_ui!(vsvg::Color, ColorWidget);
