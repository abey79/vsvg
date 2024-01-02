/// An [`egui::ComboBox`] for selecting a [`vsvg::Unit`] out of the provided list.
///
/// Returns `true` if the unit was changed.
///
/// See [`vsvg::UNITS`] and [`vsvg::SMALL_UNITS`] for a list of all available [`vsvg::Unit`].
pub fn unit_combo_box(
    ui: &mut egui::Ui,
    id_source: impl std::hash::Hash,
    unit: &mut vsvg::Unit,
    unit_choices: &[vsvg::Unit],
) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_source(id_source)
        .selected_text(unit.to_str())
        .width(50.)
        .show_ui(ui, |ui| {
            for u in unit_choices {
                changed |= ui.selectable_value(unit, *u, u.to_str()).clicked();
            }
        });

    changed
}
