pub use crate::widgets::Widget as _; // bring trait in scope, but avoid name-clash with macro
pub use crate::{
    register_widget_ui, wasm_sketch, AnimationOptions, App, Context, Grid, GridCell, HexGrid,
    HexGridCell, InfoOptions, LayoutOptions, PageSizeOptions, Result, Runner, Sketch, SketchApp,
};
pub use vsvg::{
    Color, Draw, IntoBezPath, IntoBezPathTolerance, Length, PageSize, Point, Transforms, Unit,
};
pub use whiskers_derive::{sketch_app, sketch_widget, Sketch, Widget};

#[cfg(not(target_arch = "wasm32"))]
pub use vsvg_viewer::show;

// re-exports
pub use ::serde;
pub use anyhow;
pub use egui;
pub use rand::prelude::*;
pub use vsvg_viewer;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_futures;
