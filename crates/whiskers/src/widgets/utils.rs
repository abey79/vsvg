/// An [`egui::ComboBox`] for selecting a [`vsvg::Unit`] out of the provided list.
///
/// See [`vsvg::UNITS`] for a list of all available [`Unit`].
pub fn unit_combo_box(
    ui: &mut egui::Ui,
    id_source: impl std::hash::Hash,
    unit: &mut vsvg::Unit,
    unit_choices: &[vsvg::Unit],
) -> egui::Response {
    egui::ComboBox::from_id_source(id_source)
        .selected_text(unit.to_str())
        .width(40.)
        .show_ui(ui, |ui| {
            for u in unit_choices {
                ui.selectable_value(unit, *u, u.to_str());
            }
        })
        .response
}
