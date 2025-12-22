use super::{FlattenedPath, PathDataTrait, PathMetadata, PathTrait, Point, Polyline};
use crate::Transforms;
use crate::crop::{Crop, QuadCropResult, crop_quad_bezier};
use crate::path::into_bezpath::{
    IntoBezPath, IntoBezPathTolerance, line_segment_to_bezpath, points_to_bezpath,
};
use kurbo::{Affine, BezPath, PathEl};
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;

// ======================================================================================
// The path data for `Path` is `kurbo::BezPath`.

impl Transforms for BezPath {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        self.apply_affine(*affine);
        self
    }
}

impl PathDataTrait for BezPath {
    fn bounds(&self) -> kurbo::Rect {
        kurbo::Shape::bounding_box(self)
    }

    fn start(&self) -> Option<Point> {
        if let Some(PathEl::MoveTo(pt)) = self.elements().first() {
            Some(pt.into())
        } else {
            None
        }
    }

    fn end(&self) -> Option<Point> {
        match self.elements().last()? {
            PathEl::MoveTo(pt)
            | PathEl::LineTo(pt)
            | PathEl::QuadTo(_, pt)
            | PathEl::CurveTo(_, _, pt) => Some(pt.into()),
            PathEl::ClosePath => {
                // since this may be a compound path, we must search backwards
                for el in self.elements().iter().rev() {
                    if let PathEl::MoveTo(pt) = el {
                        return Some(pt.into());
                    }
                }

                None
            }
        }
    }

    fn is_point(&self) -> bool {
        matches!(self.elements(), [PathEl::MoveTo(a), PathEl::LineTo(b)] if a == b)
    }

    fn flip(&mut self) {
        let segs: Vec<kurbo::PathSeg> = self.segments().collect();
        *self = BezPath::from_path_segments(segs.into_iter().rev().map(|seg| seg.reverse()));
    }
}

// ======================================================================================
// `Path`

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    pub data: BezPath,
    pub(crate) metadata: PathMetadata,
}

impl Transforms for Path {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        self.data.apply_affine(*affine);
        self
    }
}

impl PathTrait<BezPath> for Path {
    fn data(&self) -> &BezPath {
        &self.data
    }

    fn data_mut(&mut self) -> &mut BezPath {
        &mut self.data
    }

    fn into_data(self) -> BezPath {
        self.data
    }

    fn bounds(&self) -> kurbo::Rect {
        self.data.bounds()
    }
    fn metadata(&self) -> &PathMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut PathMetadata {
        &mut self.metadata
    }
}

impl Path {
    #[must_use]
    pub fn from_tolerance(path: impl IntoBezPathTolerance, tolerance: f64) -> Self {
        Self {
            data: path.into_bezpath_with_tolerance(tolerance),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn from_metadata(path: impl IntoBezPath, metadata: PathMetadata) -> Self {
        Self {
            data: path.into_bezpath(),
            metadata,
        }
    }

    #[must_use]
    pub fn from_tolerance_metadata(
        path: impl IntoBezPathTolerance,
        tolerance: f64,
        metadata: PathMetadata,
    ) -> Self {
        Self {
            data: path.into_bezpath_with_tolerance(tolerance),
            metadata,
        }
    }

    #[must_use]
    pub fn from_points(points: impl IntoIterator<Item = impl Into<Point>>) -> Self {
        Self::from(points_to_bezpath(points))
    }

    #[must_use]
    pub fn from_segments(points: impl IntoIterator<Item = impl Into<(Point, Point)>>) -> Self {
        Self::from(line_segment_to_bezpath(points))
    }

    pub fn from_svg(path: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            data: BezPath::from_svg(path)?,
            ..Default::default()
        })
    }

    pub fn apply_transform(&mut self, transform: Affine) {
        self.data.apply_affine(transform);
    }

    #[must_use]
    pub fn flatten(&self, tolerance: f64) -> Vec<FlattenedPath> {
        crate::trace_function!();

        let mut lines: Vec<FlattenedPath> = vec![];
        let current_line: RefCell<Polyline> = RefCell::new(Polyline::default());

        self.data.flatten(tolerance, |el| match el {
            PathEl::MoveTo(pt) => {
                if !current_line.borrow().points().is_empty() {
                    // lines.push(Polyline::from(current_line.replace(vec![])));
                    lines.push(FlattenedPath::from(
                        current_line.replace(Polyline::default()),
                    ));
                }
                current_line.borrow_mut().points_mut().push(pt.into());
            }
            PathEl::LineTo(pt) => current_line.borrow_mut().points_mut().push(pt.into()),
            PathEl::ClosePath => {
                let pt = current_line.borrow().points()[0];
                current_line.borrow_mut().points_mut().push(pt);
            }
            _ => unreachable!(),
        });

        let current_line = current_line.into_inner();
        if !current_line.points().is_empty() {
            lines.push(FlattenedPath::from(current_line));
        }

        for line in &mut lines {
            *line.metadata_mut() = self.metadata().clone();
        }

        lines
    }

    #[must_use]
    pub fn bezier_handles(&self) -> Vec<FlattenedPath> {
        crate::trace_function!();

        self.data
            .segments()
            .filter_map(|segment| match segment {
                kurbo::PathSeg::Cubic(cubic) => Some([
                    vec![cubic.p0.into(), cubic.p1.into()],
                    vec![cubic.p2.into(), cubic.p3.into()],
                ]),
                kurbo::PathSeg::Quad(quad) => Some([
                    vec![quad.p0.into(), quad.p1.into()],
                    vec![quad.p1.into(), quad.p2.into()],
                ]),
                kurbo::PathSeg::Line(_) => None,
            })
            .flatten()
            .map(FlattenedPath::from)
            .collect()
    }

    pub fn crop(
        &mut self,
        x_min: impl Into<f64>,
        y_min: impl Into<f64>,
        x_max: impl Into<f64>,
        y_max: impl Into<f64>,
    ) -> &Self {
        crate::trace_function!();

        let x_min = x_min.into();
        let y_min = y_min.into();
        let x_max = x_max.into();
        let y_max = y_max.into();

        let new_bezpath = BezPath::from_path_segments(self.data.segments().flat_map(|segment| {
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
                kurbo::PathSeg::Quad(quad) => {
                    match crop_quad_bezier(quad, x_min, y_min, x_max, y_max) {
                        QuadCropResult::Quad(quad) => vec![kurbo::PathSeg::Quad(quad)],
                        QuadCropResult::Cubic(cubic) => {
                            cubic.into_iter().map(kurbo::PathSeg::Cubic).collect()
                        }
                    }
                }
            }
        }));

        self.data = new_bezpath;
        self
    }
}

impl<T: IntoBezPath> From<T> for Path {
    fn from(path: T) -> Self {
        Self {
            data: path.into_bezpath(),
            ..Default::default()
        }
    }
}

impl From<FlattenedPath> for Path {
    fn from(path: FlattenedPath) -> Self {
        Self {
            data: path.data.into_bezpath(),
            metadata: path.metadata,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use kurbo::Line;

    #[test]
    fn test_path_crop() {
        let mut path = Path::from(Line::new((0.0, 0.0), (1.0, 1.0)));
        path.crop(0.5, 0.5, 1.5, 1.5);
        let mut it = path.data.segments();
        assert_eq!(
            it.next().unwrap(),
            kurbo::PathSeg::Line(Line::new((0.5, 0.5), (1.0, 1.0)))
        );
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_path_bounds() {
        let path = Path::from(Line::new((0.0, 0.0), (1.0, 1.0)));
        assert_eq!(path.bounds(), kurbo::Rect::new(0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn test_path_start_end() {
        let path = Path::from_svg("M 0,0 L 50,110").unwrap();
        assert_eq!(path.data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(path.data.end(), Some(Point::new(50.0, 110.0)));

        let path = Path::from_svg("M 0,0 C 50,110 50,140 60,78").unwrap();
        assert_eq!(path.data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(path.data.end(), Some(Point::new(60.0, 78.0)));

        let path = Path::from_svg("M 0,0 C 50,110 50,140 60,78 Z").unwrap();
        assert_eq!(path.data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(path.data.end(), Some(Point::new(0.0, 0.0)));

        let path = Path::from_svg("M 0,0 C 50,110 50,140 60,78 M60,43 l30,50 Z").unwrap();
        assert_eq!(path.data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(path.data.end(), Some(Point::new(60.0, 43.0)));
    }

    #[test]
    fn test_path_is_point() {
        let path = Path::from_svg("M 10,0 l 0,0").unwrap();
        assert!(path.data.is_point());

        let path = Path::from_svg("M 10,0 L 10,0").unwrap();
        assert!(path.data.is_point());
    }
}
