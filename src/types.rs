use crate::crop::Crop;
use kurbo::{BezPath, PathEl};
use std::cell::RefCell;

#[derive(Default)]
pub struct Polyline {
    pub points: Vec<[f64; 2]>,
}

impl From<Vec<[f64; 2]>> for Polyline {
    fn from(points: Vec<[f64; 2]>) -> Self {
        Self { points }
    }
}

#[derive(Default)]
pub struct Polylines {
    pub lines: Vec<Polyline>,
}

impl Polylines {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }

    fn from_bezier(path: &BezPath, tolerance: f64) -> Self {
        let mut lines: Vec<Polyline> = vec![];
        let current_line: RefCell<Vec<[f64; 2]>> = RefCell::new(vec![]);

        path.flatten(tolerance, |el| match el {
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

        Self { lines }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Polyline> {
        self.lines.iter()
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Polyline> {
        self.lines.iter_mut()
    }

    fn add_lines(&mut self, lines: Vec<Polyline>) {
        self.lines.extend(lines);
    }
}

impl IntoIterator for Polylines {
    type Item = Polyline;
    type IntoIter = std::vec::IntoIter<Polyline>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

const DEFAULT_TOLERANCE: f64 = 0.1;

pub struct Document {
    paths: Vec<BezPath>,
}

impl Document {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }

    pub fn add_path<T: kurbo::Shape>(&mut self, path: T) {
        self.paths.push(path.into_path(DEFAULT_TOLERANCE));
    }

    #[allow(dead_code)]
    pub fn add_path_with_tolerance<T: kurbo::Shape>(&mut self, path: T, tolerance: f64) {
        self.paths.push(path.into_path(tolerance));
    }

    pub fn add_paths(&mut self, mut paths: Vec<BezPath>) {
        self.paths.append(&mut paths);
    }

    pub fn flatten(&self, tolerance: f64) -> Polylines {
        self.paths
            .iter()
            .fold(Polylines::new(), |mut polylines, path| {
                polylines.add_lines(Polylines::from_bezier(path, tolerance).lines);
                polylines
            })
    }

    pub fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        fn crop_bezpath(path: BezPath, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> BezPath {
            BezPath::from_path_segments(path.segments().flat_map(|segment| {
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
            }))
        }

        Document {
            paths: self
                .paths
                .into_iter()
                .map(|path| crop_bezpath(path, x_min, y_min, x_max, y_max))
                .collect(),
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
