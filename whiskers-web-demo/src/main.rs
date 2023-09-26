#[cfg(not(target_arch = "wasm32"))]
use vsvg_sketch::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result {
    //TODO: this duplicates the code in lib.rs
    Runner::new(whiskers_web_demo::WhiskersDemoSketch::default())
        .with_page_size(PageSize::custom(12., 12., Unit::CM))
        .with_time_enabled(false)
        .run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
