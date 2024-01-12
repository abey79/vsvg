/// Widget to display [`crate::Color`] sketch parameters.
#[derive(Default)]
pub struct ColorWidget;

impl whiskers_widgets::Widget<crate::Color> for ColorWidget {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut crate::Color) -> bool {
        // empty first column
        ui.label(label);

        let mut color_components: [f32; 4] = (*value).into();
        let resp = ui.color_edit_button_rgba_unmultiplied(&mut color_components);
        if resp.changed() {
            *value = color_components.into();
        }

        resp.changed()
    }
}

whiskers_widgets::register_widget_ui!(crate::Color, ColorWidget);
