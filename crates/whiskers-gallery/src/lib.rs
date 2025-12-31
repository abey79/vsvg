//! Whiskers Gallery - A collection of interactive sketches for whisk.rs

pub mod sketches;

pub use sketches::{SketchMeta, SKETCH_MANIFEST};

// WASM entry points - one per sketch to avoid trait object complexity
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub async fn start_schotter(
        handle: &vsvg_viewer::web_handle::WebHandle,
        canvas: vsvg_viewer::exports::web_sys::HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        handle.start(canvas, super::sketches::schotter::runner()).await
    }

    #[wasm_bindgen]
    pub async fn start_hello_world(
        handle: &vsvg_viewer::web_handle::WebHandle,
        canvas: vsvg_viewer::exports::web_sys::HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        handle.start(canvas, super::sketches::hello_world::runner()).await
    }
}
