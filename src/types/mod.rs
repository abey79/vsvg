mod crop;
pub mod document;
pub mod flattened_layer;
pub mod flattened_path;
pub mod layer;
pub mod path;
pub mod transforms;

pub use document::*;
pub use document::*;
pub use flattened_path::*;
pub use layer::*;
pub use path::*;

#[derive(Default, Clone, Copy, Debug)]
pub struct PageSize {
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug)]
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
