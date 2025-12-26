pub use crate::{
    AnimationOptions, App, Context, Grid, GridCell, HexGrid, HexGridCell, InfoOptions,
    LayoutOptions, PageSizeOptions, Result, Sketch, SketchApp, wasm_sketch,
};
pub use vsvg::{
    Angle, Color, Draw, IntoBezPath, IntoBezPathTolerance, Length, PageSize, Point, Transforms,
    Unit,
};
pub use vsvg_viewer::DisplayOptions;
pub use whiskers_widgets;
pub use whiskers_widgets::Widget as _;
pub use whiskers_widgets::{Sketch, Widget, sketch_app, sketch_widget}; // bring trait in scope, but avoid name-clash with macro

#[cfg(not(target_arch = "wasm32"))]
pub use vsvg_viewer::show;

// re-exports
pub use crate::exports::{egui, serde};
pub use anyhow;
pub use rand::prelude::*;
pub use vsvg::exports::kurbo;
pub use vsvg_viewer;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_futures;
