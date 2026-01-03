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

        kurbo::flatten(&self.data, tolerance, |el| match el {
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

    /// Append another path to this one.
    ///
    /// If the endpoint of `self` and the start of `other` are within `epsilon`,
    /// the initial `MoveTo` of `other` is converted to `LineTo` to create a continuous path.
    /// Otherwise, the `MoveTo` is kept, creating a compound path with multiple subpaths.
    ///
    /// Metadata is merged (currently first path's metadata wins).
    pub fn join(&mut self, other: &Path, epsilon: f64) {
        let dominated = match (self.data.end(), other.data.start()) {
            (Some(end), Some(start)) => end.distance(&start) < epsilon,
            _ => false,
        };

        for (i, el) in other.data.elements().iter().enumerate() {
            if i == 0 {
                match el {
                    PathEl::MoveTo(pt) if dominated => {
                        self.data.push(PathEl::LineTo(*pt));
                    }
                    _ => self.data.push(*el),
                }
            } else {
                self.data.push(*el);
            }
        }

        self.metadata.merge(&other.metadata);
    }

    /// Split a compound path into its individual subpaths.
    ///
    /// Each `MoveTo` element starts a new subpath. Metadata is cloned to all
    /// resulting paths.
    ///
    /// Returns a single-element `Vec` if the path has only one subpath.
    /// Returns an empty `Vec` if the path is empty.
    ///
    /// This is useful before [`Layer::join_paths`](crate::Layer::join_paths) to
    /// maximize optimization opportunities, since `join_paths` only considers
    /// the start/end points of each path, not internal subpath endpoints.
    #[must_use]
    pub fn split(self) -> Vec<Self> {
        let elements = self.data.elements();
        if elements.is_empty() {
            return vec![];
        }

        let mut result = Vec::new();
        let mut current = BezPath::new();

        for el in elements {
            match el {
                PathEl::MoveTo(pt) => {
                    // Save current path if non-empty and start new one
                    if !current.elements().is_empty() {
                        result.push(Path {
                            data: std::mem::take(&mut current),
                            metadata: self.metadata.clone(),
                        });
                    }
                    current.push(PathEl::MoveTo(*pt));
                }
                _ => {
                    current.push(*el);
                }
            }
        }

        // Don't forget the last subpath
        if !current.elements().is_empty() {
            result.push(Path {
                data: current,
                metadata: self.metadata,
            });
        }

        result
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

    #[test]
    fn test_path_split_simple() {
        // Single subpath - returns vec with one element
        let path = Path::from_svg("M 0,0 L 10,10 L 20,0").unwrap();
        let parts = path.split();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].data.elements().len(), 3); // MoveTo + 2 LineTo
    }

    #[test]
    fn test_path_split_compound() {
        // Two subpaths
        let path = Path::from_svg("M 0,0 L 10,10 M 50,50 L 60,60").unwrap();
        let parts = path.split();
        assert_eq!(parts.len(), 2);

        // First subpath: M 0,0 L 10,10
        assert_eq!(parts[0].data.elements().len(), 2);
        assert_eq!(parts[0].data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(parts[0].data.end(), Some(Point::new(10.0, 10.0)));

        // Second subpath: M 50,50 L 60,60
        assert_eq!(parts[1].data.elements().len(), 2);
        assert_eq!(parts[1].data.start(), Some(Point::new(50.0, 50.0)));
        assert_eq!(parts[1].data.end(), Some(Point::new(60.0, 60.0)));
    }

    #[test]
    fn test_path_split_three_subpaths() {
        let path = Path::from_svg("M 0,0 L 10,0 M 20,0 L 30,0 M 40,0 L 50,0").unwrap();
        let parts = path.split();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_path_split_empty() {
        let path = Path::default();
        let parts = path.split();
        assert!(parts.is_empty());
    }

    #[test]
    fn test_path_split_with_close() {
        // Closed subpath followed by open subpath
        let path = Path::from_svg("M 0,0 L 10,0 L 10,10 Z M 50,50 L 60,60").unwrap();
        let parts = path.split();
        assert_eq!(parts.len(), 2);

        // First is closed (has ClosePath)
        assert!(
            parts[0]
                .data
                .elements()
                .iter()
                .any(|el| matches!(el, PathEl::ClosePath))
        );

        // Second is open
        assert!(
            !parts[1]
                .data
                .elements()
                .iter()
                .any(|el| matches!(el, PathEl::ClosePath))
        );
    }
}
