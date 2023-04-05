#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod crop;
pub mod document;
pub mod draw;
pub mod flattened_layer;
pub mod flattened_path;
pub mod layer;
pub mod optimization;
pub mod path;
pub mod path_index;
pub mod point;
pub mod stats;
mod svg_reader;
pub mod svg_writer;
pub mod test_utils;
pub mod transforms;

pub use document::*;
pub use draw::*;
pub use flattened_path::*;
pub use layer::*;
pub use path::*;
pub use path_index::IndexBuilder;
use std::fmt;
use std::fmt::{Display, Formatter};
pub use transforms::*;

#[derive(Default, Clone, Copy, Debug)]
pub struct PageSize {
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl Color {
    #[must_use]
    pub fn to_rgba(&self) -> u32 {
        u32::from(self.r)
            | u32::from(self.g) << 8
            | u32::from(self.b) << 16
            | u32::from(self.a) << 24
    }
}
