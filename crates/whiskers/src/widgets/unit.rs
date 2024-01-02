/// A widget for [`vsvg::Unit`].
#[derive(Default)]
pub struct UnitWidget {
    all_units: bool,
}

impl UnitWidget {
    /// Enable all units (including large ones such as [`vsvg::Unit::Km`]).
    #[must_use]
    pub fn all_units(mut self, all_units: bool) -> Self {
        self.all_units = all_units;
        self
    }
}

impl super::Widget<vsvg::Unit> for UnitWidget {
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut vsvg::Unit) -> bool {
        ui.add(egui::Label::new(label));

        ui.horizontal(|ui| {
            let units = if self.all_units {
                vsvg::UNITS
            } else {
                vsvg::SMALL_UNITS
            };

            crate::widgets::unit_combo_box(ui, label, value, units)
        })
        .inner
    }
}

crate::register_widget_ui!(vsvg::Unit, UnitWidget);
