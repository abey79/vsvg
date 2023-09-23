use crate::Sketch;
use rand::SeedableRng;

use vsvg_viewer::DocumentWidget;

pub(crate) struct SketchRunner {
    pub(crate) app: Box<dyn crate::SketchApp>,
    inited: bool,
    seed: u32,
}

impl SketchRunner {
    pub(crate) fn new(app: Box<dyn crate::SketchApp>) -> Self {
        Self {
            app,
            inited: false,
            seed: 0,
        }
    }

    fn seed_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.strong("Random Number Generator");

        ui.horizontal(|ui| {
            ui.label("seed:");
            changed = ui
                .add(egui::DragValue::new(&mut self.seed).speed(1.0))
                .changed();
        });

        ui.horizontal(|ui| {
            if ui.button("random").clicked() {
                self.seed = rand::random();
                changed = true;
            }
            if ui
                .add_enabled(self.seed != 0, egui::Button::new("prev"))
                .clicked()
            {
                self.seed = self.seed.saturating_sub(1);
                changed = true;
            }
            if ui
                .add_enabled(self.seed != u32::MAX, egui::Button::new("next"))
                .clicked()
            {
                self.seed = self.seed.saturating_add(1);
                changed = true;
            }
        });

        changed
    }
}

impl vsvg_viewer::ViewerApp for SketchRunner {
    fn update(
        &mut self,
        ctx: &egui::Context,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        let mut sketch = Sketch::new();

        let changed = egui::SidePanel::right("right_panel")
            .default_width(200.)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let changed = self.seed_ui(ui);
                    ui.separator();
                    ui.strong("Sketch Parameters");

                    egui::Grid::new("sketch_param_grid")
                        .num_columns(2)
                        .show(ui, |ui| self.app.ui(ui))
                        .inner
                        || changed
                })
            })
            .inner
            .inner;

        if changed || !self.inited {
            self.inited = true;

            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(self.seed as u64);
            self.app.update(&mut sketch, &mut rng)?;
            document_widget.set_document_data(vsvg_viewer::DocumentData::new(sketch.document()));
        }

        Ok(())
    }
}
