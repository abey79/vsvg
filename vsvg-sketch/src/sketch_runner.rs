use crate::Sketch;

use vsvg_viewer::DocumentWidget;

pub(crate) struct SketchRunner {
    pub(crate) app: Box<dyn crate::SketchApp>,
    inited: bool,
}

impl SketchRunner {
    pub(crate) fn new(app: Box<dyn crate::SketchApp>) -> Self {
        Self { app, inited: false }
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
            .show(ctx, |ui| self.app.ui(ui))
            .inner;

        if changed || !self.inited {
            self.inited = true;

            self.app.update(&mut sketch)?;
            document_widget.set_document_data(vsvg_viewer::DocumentData::new(sketch.document()));
        }

        Ok(())
    }
}
