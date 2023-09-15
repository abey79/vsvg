use super::{FlattenedPath, PathDataTrait, PathMetadata, PathTrait, Point, Polyline};
use crate::crop::Crop;
use crate::Transforms;
use kurbo::{Affine, BezPath, PathEl};
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;

const DEFAULT_TOLERANCE: f64 = 0.05;

// ======================================================================================
// The path data for `Path` is `kurbo::BezPath`.

impl Transforms for BezPath {
    fn transform(&mut self, affine: &Affine) {
        self.apply_affine(*affine);
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
    fn transform(&mut self, affine: &Affine) {
        self.data.apply_affine(*affine);
    }
}

impl PathTrait<BezPath> for Path {
    fn data(&self) -> &BezPath {
        &self.data
    }

    fn data_mut(&mut self) -> &mut BezPath {
        &mut self.data
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
    pub fn from_shape<T: kurbo::Shape>(shape: T) -> Self {
        Self::from_shape_with_tolerance(shape, DEFAULT_TOLERANCE)
    }
    pub fn from_shape_with_tolerance<T: kurbo::Shape>(shape: T, tolerance: f64) -> Self {
        Self {
            data: shape.into_path(tolerance),
            ..Default::default()
        }
    }

    pub fn from_points(points: impl IntoIterator<Item = Point>) -> Self {
        Self {
            data: points_to_bezpath(points),
            ..Default::default()
        }
    }

    pub fn from_line_segments(
        segments: impl IntoIterator<Item = impl Into<(Point, Point)>>,
    ) -> Self {
        Self {
            data: line_segment_to_bezpath(segments),
            ..Default::default()
        }
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
    pub fn control_points(&self) -> Vec<FlattenedPath> {
        self.data
            .segments()
            .filter_map(|segment| match segment {
                kurbo::PathSeg::Cubic(cubic) => Some([
                    vec![cubic.p0.into(), cubic.p1.into()],
                    vec![cubic.p2.into(), cubic.p3.into()],
                ]),
                _ => None,
            })
            .flatten()
            .map(FlattenedPath::from)
            .collect()
    }

    pub fn crop(&mut self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> &Self {
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
                kurbo::PathSeg::Quad(_) => vec![], // TODO: implement for completeness?
            }
        }));

        self.data = new_bezpath;
        self
    }
}

fn points_to_bezpath(points: impl IntoIterator<Item = impl Into<Point>>) -> BezPath {
    let mut bezpath = BezPath::new();

    let mut points = points.into_iter().map(Into::into);

    if let Some(pt) = points.next() {
        bezpath.move_to(pt);
    }

    for pt in points {
        bezpath.line_to(pt);
    }

    bezpath
}

fn line_segment_to_bezpath(
    segments: impl IntoIterator<Item = impl Into<(Point, Point)>>,
) -> BezPath {
    let segments = segments
        .into_iter()
        .map(Into::into)
        .map(|(a, b)| kurbo::PathSeg::Line(kurbo::Line::new(a, b)));

    BezPath::from_path_segments(segments)
}

impl From<BezPath> for Path {
    fn from(path: BezPath) -> Self {
        Self {
            data: path,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use kurbo::Line;
    use kurbo::PathEl::{LineTo, MoveTo};

    #[test]
    fn test_path_crop() {
        let mut path = Path::from_shape(Line::new((0.0, 0.0), (1.0, 1.0)));
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
        let path = Path::from_shape(Line::new((0.0, 0.0), (1.0, 1.0)));
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
    fn test_points_to_bezpath() {
        let points = vec![[0.0, 0.0], [10.0, 12.0], [1.0, 2.0]];

        assert_eq!(
            points_to_bezpath(points),
            BezPath::from_vec(vec![
                MoveTo(kurbo::Point::new(0.0, 0.0)),
                LineTo(kurbo::Point::new(10.0, 12.0)),
                LineTo(kurbo::Point::new(1.0, 2.0))
            ])
        );
    }

    #[test]
    fn test_points_to_bezpath_empty() {
        let points: [Point; 0] = [];
        assert!(points_to_bezpath(points).is_empty());

        let points = [Point::new(0.0, 0.0)];
        assert!(points_to_bezpath(points).is_empty());
    }

    #[test]
    fn test_line_segments_to_bezpath() {
        let segs = [
            (Point::new(0.0, 0.0), Point::new(10.0, 12.0)),
            (Point::new(1.0, 2.0), Point::new(3.0, 4.0)),
        ];
        let bezpath = line_segment_to_bezpath(segs);

        assert_eq!(
            bezpath,
            BezPath::from_vec(vec![
                MoveTo(kurbo::Point::new(0.0, 0.0)),
                LineTo(kurbo::Point::new(10.0, 12.0)),
                MoveTo(kurbo::Point::new(1.0, 2.0)),
                LineTo(kurbo::Point::new(3.0, 4.0))
            ])
        );
    }
}
