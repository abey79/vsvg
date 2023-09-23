pub mod prelude;
pub mod sketch;
mod sketch_runner;
pub mod widgets;

pub type Result = anyhow::Result<()>;

pub use sketch::Sketch;

/// This is the trait that your sketch app must implement.
pub trait App {
    fn update(&mut self, sketch: &mut Sketch) -> anyhow::Result<()>;

    //TODO:
    // - extra ui?
    // - extra CLI?
}

pub trait SketchUI {
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub trait SketchApp: App + SketchUI {}

pub fn run_default<APP: SketchApp + Default + 'static>() -> anyhow::Result<()> {
    vsvg_viewer::show_with_viewer_app(Box::new(sketch_runner::SketchRunner {
        app: Box::<APP>::default(),
    }))
}

pub fn run<APP: SketchApp + 'static>(app: APP) -> anyhow::Result<()> {
    vsvg_viewer::show_with_viewer_app(Box::new(sketch_runner::SketchRunner { app: Box::new(app) }))
}
