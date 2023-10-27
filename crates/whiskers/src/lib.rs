//! Whiskers is an interactive environment for pen-plotter generative art sketches.
//!
//! # Native sketch
//!
//! To create a whiskers sketch that will only be run natively, create a new Rust project and add
//! the whiskers crate as a dependency:
//!
//! ```bash
//! cargo add whiskers
//! ```
//!
//! Then, add the following content to the `main.rs` file:
//!
//! ```no_run
//! use whiskers::prelude::*;
//!
//! #[derive(Sketch)]
//! struct MySketch {
//!     /* add sketch parameters here */
//! }
//!
//! impl Default for MySketch {
//!     fn default() -> Self {
//!         Self {
//!             /* initialize sketch parameters to default values here */
//!         }
//!     }
//! }
//!
//! impl App for MySketch {
//!     fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
//!         // draw code goes here
//!         sketch
//!             .color(Color::DARK_RED)
//!             .rect(200., 200., 150., 50.);
//!
//!         Ok(())
//!     }
//! }
//!
//! fn main() -> Result {
//!     Runner::new(MySketch::default())
//!         .with_page_size_options(PageSize::A5H)
//!         /* add other Runner default configuration here */
//!         .run()
//! }
//! ```
//!
//! See the [`crate::Sketch`] type documentation for more information on the drawing code. See the
//! [`crate::Runner`] type documentation for more information on the available configurations.
//!
//! # Sketches with Wasm support
//!
//! For sketches that target both native and Wasm, your crate should be structured as both a library
//! and a binary. The library should contain the sketch code as well as the [`wasm_sketch!`] macro:
//!
//! ```no_run
//! // lib.rs
//!
//! use whiskers::prelude::*;
//! use whiskers::wasm_main;
//!
//! #[derive(Sketch)]
//! struct MySketch { }
//!
//! impl Default for MySketch {
//!     fn default() -> Self {
//!         Self { }
//!     }
//! }
//!
//! impl App for MySketch {
//!     fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
//!         Ok(())
//!     }
//! }
//! wasm_sketch!(Runner::new(MySketch::default()));
//! ```
//!
//! The binary should then call the [`wasm_main!`] macro:
//!
//! ```ignore
//! // main.rs
//!
//! wasm_main!(my_sketch);  // `my_sketch` is the crate name
//! ```
//!
//! Deploying the Wasm sketch additionally requires a spacial `index.html` file. See the
//! [`whiskers-web-demo`](https://github.com/abey79/vsvg/tree/master/crates/whiskers-web-demo) crate
//! for an example.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

mod context;

mod grid_helpers;

/// This module re-export all the types, traits, macros, and dependencies needed to run a sketch.
pub mod prelude;
mod runner;
mod sketch;
pub mod widgets;

pub use context::Context;
pub use grid_helpers::{
    grid::{Grid, GridCell},
    hex_grid::{cell::HexGridCell, HexGrid},
};
pub use runner::{AnimationOptions, InfoOptions, LayoutOptions, PageSizeOptions, Runner};
pub use sketch::Sketch;

/// This is a convenience alias to the [`anyhow::Result`] type, which you can use for your sketch's
/// main function.
pub type Result = anyhow::Result<()>;

/// This is the trait that your sketch app must explicitly implement. The [`App::update`] function
/// is where the sketch draw code goes.
pub trait App {
    /// Draw the sketch.
    ///
    /// This function must contain the actual draw code, using the provided [`Sketch`] and, if
    /// needed, [`crate::Context`] object.
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut context::Context) -> anyhow::Result<()>;

    //TODO:
    // - extra ui?
    // - extra CLI?
}

/// This trait is implemented by the [`whiskers_derive::Sketch`] derive macro and makes it possible
/// for the [`Runner`] to execute your sketch.s
pub trait SketchApp: App {
    /// The name of the sketch, used the window title, the default output file name, and persistent
    /// settings.
    fn name(&self) -> String;

    /// Draw the UI for the sketch, return whether the sketch should be updated.
    ///
    /// This function is generated by the [`Sketch`] derive macro.
    fn ui(&mut self, ui: &mut egui::Ui) -> bool;
}

/// Declare the main entry point for wasm builds.
///
/// Note: this macro requires `use whiskers::prelude::*;` to be present in the module.
#[macro_export]
macro_rules! wasm_sketch {
    ($t: expr) => {
        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub async fn start(
            handle: &vsvg_viewer::web_handle::WebHandle,
            canvas_id: &str,
        ) -> std::result::Result<(), wasm_bindgen::JsValue> {
            handle.start(canvas_id, $t).await
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub fn main_func() -> Result {
            $t.run()
        }

        #[cfg(target_arch = "wasm32")]
        pub fn main_func() -> Result {
            Ok(())
        }
    };
}

/// Declare the binary entry point in Wasm-ready sketch crates.
///
/// For crates which targets both native and Wasm, this macro should be used to implement the
/// crate's binary (i.e. `main.rs`):
///
/// ```ignore
/// // main.rs
///
/// whiskers::wasm_main!(my_sketch);  // `my_sketch` is the crate name
/// ```
#[macro_export]
macro_rules! wasm_main {
    ($lib: ident) => {
        fn main() -> whiskers::prelude::anyhow::Result<()> {
            $lib::main_func()
        }
    };
}
