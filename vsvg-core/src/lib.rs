//! `vsvg-core` is a library crate to manipulate vector graphics, with a focus on SVG and
//! pen-plotter applications. It's inspired upon [`vpype`](https://github.com/abey79/vpype), the
//! Swiss-Army-knife command-line tool for plotter vector graphics.
//!
//! TODO: complete this documentation
//!
//! # Example
//!
//! ```
//! use vsvg_core::{Document, DocumentTrait, Layer, Path};
//!
//! let mut doc = Document::default();
//!
//! // todo
//! // - create path from anything, add metadata
//! // - add Layer::push_path + add layer
//! // - add Document::push_path
//!
//!```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod crop;
pub mod document;
pub mod draw;
pub mod layer;
pub mod optimization;
pub mod path;
pub mod path_index;
pub mod stats;
mod svg_reader;
pub mod svg_writer;
pub mod test_utils;
pub mod transforms;

pub use document::*;
pub use draw::*;
pub use layer::*;
pub use path::*;
pub use path_index::IndexBuilder;
use std::fmt;
use std::fmt::{Display, Formatter};
pub use transforms::*;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
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

impl Color {
    #[must_use]
    pub fn to_rgb_string(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    #[must_use]
    pub fn opacity(&self) -> f32 {
        f32::from(self.a) / 255.0
    }
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
    pub const WHITE: Self = Self::gray(0xFF);
    pub const BLACK: Self = Self::gray(0);

    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }

    #[must_use]
    pub const fn gray(v: u8) -> Self {
        Self {
            r: v,
            g: v,
            b: v,
            a: 0xFF,
        }
    }

    #[must_use]
    pub const fn to_rgba(&self) -> u32 {
        self.r as u32 | (self.g as u32) << 8 | (self.b as u32) << 16 | (self.a as u32) << 24
    }
}
