mod crop;
pub mod document;
pub mod flattened_layer;
pub mod flattened_path;
pub mod layer;
pub mod path;

pub use document::*;
pub use document::*;
pub use flattened_path::*;
pub use layer::*;
pub use path::*;

// =================================================================================================
// Polylines

//TODO: might need to put that somewhere

//
// pub fn iter(&self) -> impl Iterator<Item = &Polyline> {
//     self.lines.iter()
// }
//
// #[allow(dead_code)]
// pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Polyline> {
//     self.lines.iter_mut()
// }
//
// fn append(&mut self, other: &mut Self) {
//     self.lines.append(&mut other.lines);
// }

// impl IntoIterator for Polylines {
//     type Item = Polyline;
//     type IntoIter = std::vec::IntoIter<Polyline>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self.lines.into_iter()
//     }
// }

// =================================================================================================
// Page Size

#[derive(Default, Clone, Copy, Debug)]
pub struct PageSize {
    pub w: f64,
    pub h: f64,
}

// =================================================================================================
// Color

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

#[cfg(test)]
mod tests {
    use super::*;
}
