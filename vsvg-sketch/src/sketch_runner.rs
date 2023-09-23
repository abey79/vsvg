use crate::Sketch;

use vsvg_viewer::DocumentWidget;

pub(crate) struct SketchRunner {
    pub(crate) app: Box<dyn crate::SketchApp>,
}

impl vsvg_viewer::ViewerApp for SketchRunner {
    fn update(
        &mut self,
        ctx: &egui::Context,
        document_widget: &mut DocumentWidget,
    ) -> anyhow::Result<()> {
        let mut sketch = Sketch::new();
        self.app.update(&mut sketch)?;
        document_widget.set_document_data(vsvg_viewer::DocumentData::new(sketch.document()));

        //FIXME: this must be smarter
        ctx.request_repaint();

        egui::SidePanel::right("right_panel")
            .default_width(200.)
            .show(ctx, |ui| {
                self.app.ui(ui);
            });
        Ok(())
    }
}
