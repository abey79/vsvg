use crate::Sketch;

use vsvg_viewer::DocumentWidget;

pub trait SketchApp {
    fn update(&mut self, _sketch: &mut Sketch) -> anyhow::Result<()>;
}

pub struct SketchRunner {
    app: Box<dyn SketchApp>,
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
        Ok(())
    }
}

impl SketchRunner {
    pub fn new(app: impl SketchApp + 'static) -> Self {
        Self {
            app: Box::new(app),
            //params: vec![],
        }
    }

    pub fn run(self) -> anyhow::Result<()> {
        vsvg_viewer::show_with_viewer_app(Box::new(self))?;

        Ok(())
    }
}

pub trait UIParameter {
    fn ui(&mut self, ui: &mut egui::Ui);
}
