//! Whiskers Gallery - A collection of interactive sketches for whisk.rs

pub mod sketches;
mod generated;

pub use generated::SKETCH_MANIFEST;
#[cfg(not(target_arch = "wasm32"))]
pub use generated::render_sketch;
pub use sketches::SketchMeta;
