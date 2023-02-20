use crate::crop::Crop;
use kurbo::{BezPath, PathEl};
use std::cell::RefCell;

const DEFAULT_TOLERANCE: f64 = 0.1;

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

// =================================================================================================
// Path

#[derive(Clone, Debug)]
pub struct Path {
    pub bezpath: BezPath,
    pub color: Color,
    pub stroke_width: f64,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            bezpath: BezPath::new(),
            color: Color::default(),
            stroke_width: 1.0,
        }
    }
}

impl Path {
    #[allow(dead_code)]
    pub fn from_shape<T: kurbo::Shape>(path: T) -> Self {
        Self::from_shape_with_tolerance(path, DEFAULT_TOLERANCE)
    }

    #[allow(dead_code)]
    pub fn from_shape_with_tolerance<T: kurbo::Shape>(path: T, tolerance: f64) -> Self {
        Self {
            bezpath: path.into_path(tolerance),
            ..Default::default()
        }
    }

    pub fn flatten(&self, tolerance: f64) -> Polylines {
        Polylines::from_path(self, tolerance)
    }

    fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        let new_bezpath =
            BezPath::from_path_segments(self.bezpath.segments().flat_map(|segment| {
                match segment {
                    kurbo::PathSeg::Line(line) => line
                        .crop(x_min, y_min, x_max, y_max)
                        .into_iter()
                        .map(kurbo::PathSeg::Line)
                        .collect(),
                    kurbo::PathSeg::Cubic(cubic) => cubic
                        .crop(x_min, y_min, x_max, y_max)
                        .into_iter()
                        .map(kurbo::PathSeg::Cubic)
                        .collect(),
                    kurbo::PathSeg::Quad(_) => vec![], // TODO: implement for completeness
                }
            }));

        Self {
            bezpath: new_bezpath,
            ..self
        }
    }
}

// =================================================================================================
// Layer

#[derive(Default, Clone, Debug)]
pub struct Layer {
    pub paths: Vec<Path>,
}

impl Layer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn flatten(&self, tolerance: f64) -> Polylines {
        self.paths
            .iter()
            .fold(Polylines::new(), |mut polylines, path| {
                polylines.append(&mut path.flatten(tolerance));
                polylines
            })
    }

    fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        Self {
            paths: self
                .paths
                .into_iter()
                .map(|path| path.crop(x_min, y_min, x_max, y_max))
                .collect(),
        }
    }
}

// =================================================================================================
// Document

#[derive(Default, Clone, Debug)]
pub struct Document {
    pub layers: Vec<Layer>,
    pub page_size: Option<PageSize>,
}

impl Document {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn new_with_page_size(page_size: PageSize) -> Self {
        Self {
            page_size: Some(page_size),
            ..Default::default()
        }
    }

    pub fn flatten(&self, tolerance: f64) -> Polylines {
        self.layers
            .iter()
            .fold(Polylines::new(), |mut polylines, layer| {
                polylines.append(&mut layer.flatten(tolerance));
                polylines
            })
    }

    pub fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        Self {
            layers: self
                .layers
                .into_iter()
                .map(|layer| layer.crop(x_min, y_min, x_max, y_max))
                .collect(),
            ..self
        }
    }
}

// -------------------------------------------------------------------------------------------------

// =================================================================================================
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polylines() {
        let polylines = Polylines::new();
        assert_eq!(polylines.lines.len(), 0);
    }
}
