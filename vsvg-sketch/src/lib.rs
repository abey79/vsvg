mod app;
pub mod prelude;
pub mod sketch;

pub type Result = anyhow::Result<()>;

pub use app::{SketchApp, SketchRunner};
pub use sketch::Sketch;
