use vsvg::{PageSize, Unit};
use whiskers_widgets::collapsing_header;

/// Controls the page size feature of the runner.
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PageSizeOptions {
    /// The configured page size.
    pub(crate) page_size: PageSize,

    /// Whether the page size is locked or not.
    #[serde(skip)]
    pub(crate) locked: bool,
}

impl From<PageSize> for PageSizeOptions {
    fn from(page_size: PageSize) -> Self {
        Self {
            page_size,
            locked: false,
        }
    }
}

impl Default for PageSizeOptions {
    fn default() -> Self {
        Self {
            page_size: PageSize::A4V,
            locked: false,
        }
    }
}

impl PageSizeOptions {
    /// Lock the page size to the provided value.
    ///
    /// The page size will not be editable in the UI.
    pub fn locked(page_size: PageSize) -> Self {
        Self {
            page_size,
            locked: true,
        }
    }
}

impl PageSizeOptions {
    #[must_use]
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        collapsing_header(ui, "Page Size", self.page_size.to_string(), true, |ui| {
            if self.locked {
                ui.label(format!("Locked to {}", self.page_size));
                return false;
            }

            let mut new_page_size = self.page_size;
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("format:");

                egui::ComboBox::from_id_salt("sketch_page_size")
                    .selected_text(new_page_size.to_format().unwrap_or("Custom"))
                    .width(120.)
                    .show_ui(ui, |ui| {
                        let orig = if matches!(new_page_size, PageSize::Custom(..)) {
                            new_page_size
                        } else {
                            PageSize::Custom(new_page_size.w(), new_page_size.h(), Unit::Px)
                        };
                        ui.selectable_value(&mut new_page_size, orig, "Custom");

                        ui.separator();

                        for page_size in &vsvg::PAGE_SIZES {
                            ui.selectable_value(
                                &mut new_page_size,
                                *page_size,
                                page_size.to_string(),
                            );
                        }
                    });

                if ui.button("flip").clicked() {
                    new_page_size = new_page_size.flip();
                }
            });

            new_page_size = if let PageSize::Custom(mut w, mut h, mut unit) = new_page_size {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut w)
                            .speed(1.0)
                            .range(0.0..=f64::MAX),
                    );

                    ui.label("x");
                    ui.add(
                        egui::DragValue::new(&mut h)
                            .speed(1.0)
                            .range(0.0..=f64::MAX),
                    );

                    let orig_unit = unit;
                    vsvg::ui::unit_combo_box(
                        ui,
                        "sketch_page_size_unit",
                        &mut unit,
                        vsvg::SMALL_UNITS,
                    );

                    w = unit.convert_from(&orig_unit, w);
                    h = unit.convert_from(&orig_unit, h);
                });

                PageSize::Custom(w, h, unit)
            } else {
                ui.label(format!(
                    "{:.1}x{:.1} mm",
                    Unit::Mm.convert(new_page_size.w()),
                    Unit::Mm.convert(new_page_size.h()),
                ));

                new_page_size
            };

            if new_page_size != self.page_size {
                self.page_size = new_page_size;
                changed = true;
            }

            changed
        })
        .unwrap_or(false)
    }
}
