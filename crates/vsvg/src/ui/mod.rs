//! UI utilities
//!
//! UI utilities used by various crates. Require the `egui` feature to be enabled.

mod list_item;

pub use list_item::*;

/// An [`egui::ComboBox`] for selecting a [`crate::Unit`] out of the provided list.
///
/// Returns `true` if the unit was changed.
///
/// See [`crate::UNITS`] and [`crate::SMALL_UNITS`] for a list of all available [`crate::Unit`].
pub fn unit_combo_box(
    ui: &mut egui::Ui,
    id_source: impl std::hash::Hash,
    unit: &mut crate::Unit,
    unit_choices: &[crate::Unit],
) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id_source)
        .selected_text(unit.to_str())
        .width(50.)
        .show_ui(ui, |ui| {
            for u in unit_choices {
                changed |= ui.selectable_value(unit, *u, u.to_str()).clicked();
            }
        });

    changed
}
