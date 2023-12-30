//! `vsvg` is a library crate to manipulate vector graphics, with a focus on SVG and
//! pen-plotter applications. It's inspired upon [`vpype`](https://github.com/abey79/vpype), the
//! Swiss-Army-knife command-line tool for plotter vector graphics.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod catmull_rom;
mod color;
mod crop;
mod document;
mod layer;
mod length;
mod optimization;
mod page_size;
mod path;
mod path_index;
mod stats;
mod svg;
mod test_utils;
mod traits;
mod unit;

pub use crate::svg::*;
pub use color::*;

pub use catmull_rom::*;
pub use crop::*;
pub use document::*;
pub use layer::*;
pub use length::*;
#[allow(unused_imports)]
pub use optimization::*;
pub use page_size::*;
pub use path::*;
pub use path_index::*;
pub use stats::*;
pub use traits::*;
pub use unit::*;

// re-export for the `trace` macro
#[cfg(feature = "puffin")]
pub use puffin;

// re-export
#[cfg(feature = "geo")]
pub use ::geo;

#[macro_export]
macro_rules! trace_function {
    () => {
        #[cfg(feature = "puffin")]
        $crate::puffin::profile_function!();
    };
}

#[macro_export]
macro_rules! trace_scope {
    ($id:expr) => {
        #[cfg(feature = "puffin")]
        $crate::puffin::profile_scope!($id);
    };
    ($id:expr, $data:expr) => {
        #[cfg(feature = "puffin")]
        $crate::puffin::profile_scope!($id, $data);
    };
}
