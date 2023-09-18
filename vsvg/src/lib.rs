//! `vsvg` is a library crate to manipulate vector graphics, with a focus on SVG and
//! pen-plotter applications. It's inspired upon [`vpype`](https://github.com/abey79/vpype), the
//! Swiss-Army-knife command-line tool for plotter vector graphics.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod crop;
pub mod document;
pub mod layer;
pub mod optimization;
pub mod path;
pub mod path_index;
pub mod stats;
mod svg_reader;
pub mod svg_writer;
pub mod test_utils;
mod traits;

pub use document::*;

pub use layer::*;
pub use path::*;
pub use path_index::IndexBuilder;
use std::fmt;
use std::fmt::{Display, Formatter};
pub use traits::*;

pub struct Units;

impl Units {
    pub const PX: f64 = 1.0;
    pub const INCH: f64 = 96.0;
    pub const FT: f64 = 12.0 * 96.0;
    pub const YARD: f64 = 36.0 * 96.0;
    pub const MI: f64 = 1760.0 * 36.0 * 96.0;
    pub const MM: f64 = 96.0 / 25.4;
    pub const CM: f64 = 96.0 / 2.54;
    pub const M: f64 = 100.0 * 96.0 / 2.54;
    pub const KM: f64 = 100_000.0 * 96.0 / 2.54;
    pub const PC: f64 = 16.0;
    pub const PT: f64 = 96.0 / 72.0;
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct PageSize {
    pub w: f64,
    pub h: f64,
}

macro_rules! mm {
    ($x:expr) => {
        ($x) * 96.0 / 25.4
    };
}

impl PageSize {
    pub const A6: Self = Self::new(mm!(105.0), mm!(148.0));
    pub const A5: Self = Self::new(mm!(148.0), mm!(210.0));
    pub const A4: Self = Self::new(mm!(210.0), mm!(297.0));
    pub const A3: Self = Self::new(mm!(297.0), mm!(420.0));
    pub const A2: Self = Self::new(mm!(420.0), mm!(594.0));
    pub const A1: Self = Self::new(mm!(594.0), mm!(841.0));
    pub const A0: Self = Self::new(mm!(841.0), mm!(1189.0));
    pub const LETTER: Self = Self::new(mm!(215.9), mm!(279.4));
    pub const LEGAL: Self = Self::new(mm!(215.9), mm!(355.6));
    pub const EXECUTIVE: Self = Self::new(mm!(185.15), mm!(266.7));
    pub const TABLOID: Self = Self::new(mm!(279.4), mm!(431.8));

    #[must_use]
    pub const fn new(w: f64, h: f64) -> Self {
        Self { w, h }
    }
}

// macro to convert a float literal from mm to pixels

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const DARK_GRAY: Self = Self::rgb(96, 96, 96);
    pub const GRAY: Self = Self::rgb(160, 160, 160);
    pub const LIGHT_GRAY: Self = Self::rgb(220, 220, 220);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BROWN: Self = Self::rgb(165, 42, 42);
    pub const DARK_RED: Self = Self::rgb(0x8B, 0, 0);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const LIGHT_RED: Self = Self::rgb(255, 128, 128);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const LIGHT_YELLOW: Self = Self::rgb(255, 255, 0xE0);
    pub const KHAKI: Self = Self::rgb(240, 230, 140);
    pub const DARK_GREEN: Self = Self::rgb(0, 0x64, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const LIGHT_GREEN: Self = Self::rgb(0x90, 0xEE, 0x90);
    pub const DARK_BLUE: Self = Self::rgb(0, 0, 0x8B);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const LIGHT_BLUE: Self = Self::rgb(0xAD, 0xD8, 0xE6);
    pub const GOLD: Self = Self::rgb(255, 215, 0);

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

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn with_opacity(&self, opacity: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: (opacity * 255.0) as u8,
        }
    }

    #[must_use]
    pub const fn to_rgba(&self) -> u32 {
        self.r as u32 | (self.g as u32) << 8 | (self.b as u32) << 16 | (self.a as u32) << 24
    }

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
