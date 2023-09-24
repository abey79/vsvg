//! `vsvg` is a library crate to manipulate vector graphics, with a focus on SVG and
//! pen-plotter applications. It's inspired upon [`vpype`](https://github.com/abey79/vpype), the
//! Swiss-Army-knife command-line tool for plotter vector graphics.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod color;
mod crop;
pub mod document;
pub mod layer;
pub mod optimization;
mod page_size;
pub mod path;
pub mod path_index;
pub mod stats;
mod svg_reader;
pub mod svg_writer;
pub mod test_utils;
mod traits;
mod unit;

pub use color::*;
pub use document::*;
pub use layer::*;
pub use page_size::*;
pub use path::*;
pub use path_index::IndexBuilder;
pub use traits::*;
pub use unit::*;
