mod crop;
pub mod document;
pub mod layer;
pub mod path;

pub use document::Document;
pub use layer::Layer;
pub use path::Path;

use kurbo::PathEl;
use std::cell::RefCell;

// =================================================================================================
// Polyline
//
// Used for display only. Has subset of metadata related to display. Should ultimately be moved to
// a standalone viewer crate.

#[derive(Default)]
pub struct Polyline {
    pub points: Vec<[f64; 2]>,
    pub color: Color,
    pub stroke_width: f64,
}

impl From<Vec<[f64; 2]>> for Polyline {
    fn from(points: Vec<[f64; 2]>) -> Self {
        Self {
            points,
            ..Default::default()
        }
    }
}

// =================================================================================================
// Polylines

#[derive(Default)]
pub struct Polylines {
    pub lines: Vec<Polyline>,
}

impl Polylines {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }

    fn from_path(path: &Path, tolerance: f64) -> Self {
        let mut lines: Vec<Polyline> = vec![];
        let current_line: RefCell<Vec<[f64; 2]>> = RefCell::new(vec![]);

        path.bezpath.flatten(tolerance, |el| match el {
            PathEl::MoveTo(pt) => {
                if !current_line.borrow().is_empty() {
                    lines.push(Polyline::from(current_line.replace(vec![])));
                }
                current_line.borrow_mut().push([pt.x, pt.y]);
            }
            PathEl::LineTo(pt) => current_line.borrow_mut().push([pt.x, pt.y]),
            PathEl::ClosePath => {
                let pt = current_line.borrow()[0];
                current_line.borrow_mut().push(pt)
            }
            _ => unreachable!(),
        });

        let current_line = current_line.into_inner();
        if !current_line.is_empty() {
            lines.push(Polyline::from(current_line));
        }

        for line in &mut lines {
            line.color = path.color;
            line.stroke_width = path.stroke_width
        }

        Self { lines }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Polyline> {
        self.lines.iter()
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Polyline> {
        self.lines.iter_mut()
    }

    fn append(&mut self, other: &mut Self) {
        self.lines.append(&mut other.lines);
    }
}

impl IntoIterator for Polylines {
    type Item = Polyline;
    type IntoIter = std::vec::IntoIter<Polyline>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

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

    #[test]
    fn test_polylines() {
        let polylines = Polylines::new();
        assert_eq!(polylines.lines.len(), 0);
    }
}
