/// A widget for [`crate::Unit`].
#[derive(Default)]
pub struct UnitWidget {
    all_units: bool,
}

impl UnitWidget {
    /// Enable all units (including large ones such as [`crate::Unit::Km`]).
    #[must_use]
    pub fn all_units(mut self, all_units: bool) -> Self {
        self.all_units = all_units;
        self
    }
}

impl whiskers_widgets::Widget<crate::Unit> for UnitWidget {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut crate::Unit) -> bool {
        ui.add(egui::Label::new(label));

        ui.horizontal(|ui| {
            let units = if self.all_units {
                crate::UNITS
            } else {
                crate::SMALL_UNITS
            };

            crate::ui::unit_combo_box(ui, label, value, units)
        })
        .inner
    }
}

whiskers_widgets::register_widget_ui!(crate::Unit, UnitWidget);
