//! Trait and implementation to support flexible conversion to `kurbo::BezPath`.

use crate::{Point, Polyline, DEFAULT_TOLERANCE};

use kurbo::{BezPath, PathEl, PathSeg};

/// Provided a tolerance, can be converted into a `BezPath`.
pub trait IntoBezPathTolerance {
    fn into_bezpath_with_tolerance(self, tolerance: f64) -> BezPath;
}

/// Can be converted into a `BezPath`.
///
/// This is blanket-implemented using [`IntoBezPathTolerance`]. Do not implement this trait
/// directly.
pub trait IntoBezPath {
    fn into_bezpath(self) -> BezPath;
}

impl<T: IntoBezPathTolerance> IntoBezPath for T {
    fn into_bezpath(self) -> BezPath {
        <Self as IntoBezPathTolerance>::into_bezpath_with_tolerance(self, DEFAULT_TOLERANCE)
    }
}

impl IntoBezPathTolerance for &[Point] {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
        points_to_bezpath(self.iter().copied())
    }
}

impl<const N: usize> IntoBezPathTolerance for [Point; N] {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
        points_to_bezpath(self.iter().copied())
    }
}

impl IntoBezPathTolerance for &Vec<Point> {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
        points_to_bezpath(self.iter().copied())
    }
}

impl IntoBezPathTolerance for &[(Point, Point)] {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
        line_segment_to_bezpath(self.iter().copied())
    }
}

impl<const N: usize> IntoBezPathTolerance for [(Point, Point); N] {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
        line_segment_to_bezpath(self)
    }
}

// Note: we can't use a blanket implementation on `kurbo::Shape`, so let's unroll on the available
// types.

macro_rules! kurbo_shape_into_bezpath {
    ($t:ty) => {
        impl IntoBezPathTolerance for $t {
            fn into_bezpath_with_tolerance(self, tolerance: f64) -> BezPath {
                <$t as kurbo::Shape>::into_path(self, tolerance)
            }
        }
    };
}

kurbo_shape_into_bezpath!(kurbo::PathSeg);
kurbo_shape_into_bezpath!(kurbo::Arc);
kurbo_shape_into_bezpath!(kurbo::BezPath);
kurbo_shape_into_bezpath!(kurbo::Circle);
kurbo_shape_into_bezpath!(kurbo::CircleSegment);
kurbo_shape_into_bezpath!(kurbo::CubicBez);
kurbo_shape_into_bezpath!(kurbo::Ellipse);
kurbo_shape_into_bezpath!(kurbo::Line);
kurbo_shape_into_bezpath!(kurbo::QuadBez);
kurbo_shape_into_bezpath!(kurbo::Rect);
kurbo_shape_into_bezpath!(kurbo::RoundedRect);

impl IntoBezPathTolerance for Polyline {
    fn into_bezpath_with_tolerance(self, tolerance: f64) -> BezPath {
        self.points().into_bezpath_with_tolerance(tolerance)
    }
}

#[cfg(feature = "geo")]
pub mod geo_impl {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    impl IntoBezPathTolerance for geo::Geometry<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            match self {
                geo::Geometry::Point(pt) => BezPath::from_vec(vec![
                    PathEl::MoveTo((pt.x(), pt.y()).into()),
                    PathEl::LineTo((pt.x(), pt.y()).into()),
                ]),
                geo::Geometry::MultiPoint(mp) => BezPath::from_vec(
                    mp.into_iter()
                        .flat_map(|pt| {
                            [
                                PathEl::MoveTo((pt.x(), pt.y()).into()),
                                PathEl::LineTo((pt.x(), pt.y()).into()),
                            ]
                        })
                        .collect(),
                ),
                geo::Geometry::Line(line) => {
                    BezPath::from_path_segments(std::iter::once(PathSeg::Line(kurbo::Line::new(
                        (line.start.x, line.start.y),
                        (line.end.x, line.end.y),
                    ))))
                }

                geo::Geometry::LineString(pts) => linestring_to_path_el(pts).collect::<BezPath>(),
                geo::Geometry::MultiLineString(mls) => mls
                    .into_iter()
                    .flat_map(linestring_to_path_el)
                    .collect::<BezPath>(),

                geo::Geometry::Polygon(poly) => {
                    let (exterior, interiors) = poly.into_inner();

                    linestring_to_path_el(exterior)
                        .chain(interiors.into_iter().flat_map(linestring_to_path_el))
                        .collect::<BezPath>()
                }

                geo::Geometry::MultiPolygon(mp) => mp
                    .into_iter()
                    .flat_map(|poly| {
                        let (exterior, interiors) = poly.into_inner();

                        linestring_to_path_el(exterior)
                            .chain(interiors.into_iter().flat_map(linestring_to_path_el))
                    })
                    .collect::<BezPath>(),

                geo::Geometry::GeometryCollection(coll) => coll
                    .into_iter()
                    .flat_map(IntoBezPath::into_bezpath)
                    .collect(),
                geo::Geometry::Rect(rect) => BezPath::from_vec(vec![
                    PathEl::MoveTo((rect.min().x, rect.min().y).into()),
                    PathEl::LineTo((rect.min().x, rect.max().y).into()),
                    PathEl::LineTo((rect.max().x, rect.max().y).into()),
                    PathEl::LineTo((rect.max().x, rect.min().y).into()),
                    PathEl::ClosePath,
                ]),
                geo::Geometry::Triangle(tri) => BezPath::from_vec(vec![
                    PathEl::MoveTo((tri.0.x, tri.0.y).into()),
                    PathEl::LineTo((tri.1.x, tri.1.y).into()),
                    PathEl::LineTo((tri.2.x, tri.2.y).into()),
                    PathEl::ClosePath,
                ]),
            }
        }
    }

    macro_rules! geo_object_into_bezpath {
        ( $ t: ty) => {
            impl IntoBezPathTolerance for $t {
                fn into_bezpath_with_tolerance(self, tolerance: f64) -> BezPath {
                    let geom: ::geo::Geometry = self.into();
                    geom.into_bezpath_with_tolerance(tolerance)
                }
            }
        };
    }

    geo_object_into_bezpath!(geo::Point<f64>);
    geo_object_into_bezpath!(geo::Line<f64>);
    geo_object_into_bezpath!(geo::LineString<f64>);
    geo_object_into_bezpath!(geo::Polygon<f64>);
    geo_object_into_bezpath!(geo::MultiPoint<f64>);
    geo_object_into_bezpath!(geo::MultiLineString<f64>);
    geo_object_into_bezpath!(geo::MultiPolygon<f64>);
    geo_object_into_bezpath!(geo::Rect<f64>);
    geo_object_into_bezpath!(geo::Triangle<f64>);

    pub(super) fn linestring_to_path_el(ls: geo::LineString<f64>) -> impl Iterator<Item = PathEl> {
        let closed = ls.is_closed();
        let len = ls.0.len();

        ls.into_iter().enumerate().map(move |(i, pt)| {
            if i == 0 {
                PathEl::MoveTo(coord_to_point(pt).into())
            } else if i == len - 1 && closed {
                PathEl::ClosePath
            } else {
                PathEl::LineTo(coord_to_point(pt).into())
            }
        })
    }

    #[inline]
    pub(super) fn coord_to_point(c: geo::Coord<f64>) -> Point {
        (c.x, c.y).into()
    }
}

pub(crate) fn points_to_bezpath(
    points: impl IntoIterator<Item = impl Into<Point>>,
) -> kurbo::BezPath {
    let mut bezpath = kurbo::BezPath::new();

    let mut points = points.into_iter().map(Into::into);

    if let Some(pt) = points.next() {
        bezpath.move_to(pt);
    }

    for pt in points {
        bezpath.line_to(pt);
    }

    bezpath
}

pub(crate) fn line_segment_to_bezpath(
    segments: impl IntoIterator<Item = impl Into<(Point, Point)>>,
) -> BezPath {
    let segments = segments
        .into_iter()
        .map(Into::into)
        .map(|(a, b)| kurbo::PathSeg::Line(kurbo::Line::new(a, b)));

    BezPath::from_path_segments(segments)
}
#[cfg(test)]
mod test {
    use super::geo_impl::*;
    use super::*;

    #[test]
    fn test_linestring_to_path_el() {
        // empty case
        assert_eq!(
            linestring_to_path_el(geo::LineString(vec![])).collect::<Vec<_>>(),
            vec![]
        );

        // single point, should be just a single MoveTo
        assert_eq!(
            linestring_to_path_el(geo::LineString(vec![(0., 0.).into()])).collect::<Vec<_>>(),
            vec![PathEl::MoveTo((0., 0.).into())]
        );

        // three points, not closed
        assert_eq!(
            linestring_to_path_el(geo::LineString(vec![
                (0., 0.).into(),
                (1., 1.).into(),
                (2., 2.).into()
            ]))
            .collect::<Vec<_>>(),
            vec![
                PathEl::MoveTo((0., 0.).into()),
                PathEl::LineTo((1., 1.).into()),
                PathEl::LineTo((2., 2.).into()),
            ]
        );

        // three points, closed
        assert_eq!(
            linestring_to_path_el(geo::LineString(vec![
                (0., 0.).into(),
                (1., 1.).into(),
                (2., 2.).into(),
                (0., 0.).into(),
            ]))
            .collect::<Vec<_>>(),
            vec![
                PathEl::MoveTo((0., 0.).into()),
                PathEl::LineTo((1., 1.).into()),
                PathEl::LineTo((2., 2.).into()),
                PathEl::ClosePath,
            ]
        );

        // three points, closed
        let mut ls = geo::LineString(vec![(0., 0.).into(), (1., 1.).into(), (2., 2.).into()]);
        ls.close();

        assert_eq!(
            linestring_to_path_el(ls).collect::<Vec<_>>(),
            vec![
                PathEl::MoveTo((0., 0.).into()),
                PathEl::LineTo((1., 1.).into()),
                PathEl::LineTo((2., 2.).into()),
                PathEl::ClosePath,
            ]
        );
    }

    #[test]
    fn test_points_to_bezpath() {
        let points = vec![[0.0, 0.0], [10.0, 12.0], [1.0, 2.0]];

        assert_eq!(
            points_to_bezpath(points),
            BezPath::from_vec(vec![
                PathEl::MoveTo(kurbo::Point::new(0.0, 0.0)),
                PathEl::LineTo(kurbo::Point::new(10.0, 12.0)),
                PathEl::LineTo(kurbo::Point::new(1.0, 2.0))
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
                PathEl::MoveTo(kurbo::Point::new(0.0, 0.0)),
                PathEl::LineTo(kurbo::Point::new(10.0, 12.0)),
                PathEl::MoveTo(kurbo::Point::new(1.0, 2.0)),
                PathEl::LineTo(kurbo::Point::new(3.0, 4.0))
            ])
        );
    }
}
