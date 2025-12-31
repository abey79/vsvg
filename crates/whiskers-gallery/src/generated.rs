// AUTO-GENERATED FROM sketches.toml - DO NOT EDIT
// Run `cargo build -p whiskers-gallery` to regenerate

use crate::sketches::SketchMeta;

/// Registry of all available sketches.
pub static SKETCH_MANIFEST: &[SketchMeta] = &[
    SketchMeta {
        id: "schotter",
        name: "Schotter",
        description: "Recreation of Georg Nees' classic 1968-1970 generative art piece",
        author: "Antoine Beyeler",
    },
    SketchMeta {
        id: "hello_world",
        name: "Hello World",
        description: "A simple introductory sketch demonstrating basic whiskers usage",
        author: "Antoine Beyeler",
    },
];

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub async fn start_schotter(
        handle: &vsvg_viewer::web_handle::WebHandle,
        canvas: vsvg_viewer::exports::web_sys::HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        handle.start(canvas, crate::sketches::schotter::runner()).await
    }

    #[wasm_bindgen]
    pub async fn start_hello_world(
        handle: &vsvg_viewer::web_handle::WebHandle,
        canvas: vsvg_viewer::exports::web_sys::HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        handle.start(canvas, crate::sketches::hello_world::runner()).await
    }
}
