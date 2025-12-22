//! Trait and implementation to support flexible conversion to `kurbo::BezPath`.

use crate::{DEFAULT_TOLERANCE, Point, Polyline};

use kurbo::BezPath;

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

impl IntoBezPathTolerance for &[(f64, f64)] {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
        points_to_bezpath(self.iter().copied())
    }
}

impl IntoBezPathTolerance for Vec<(f64, f64)> {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
        points_to_bezpath(self)
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

    use kurbo::{PathEl, PathSeg};

    impl IntoBezPathTolerance for &geo::Point<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            BezPath::from_vec(vec![
                PathEl::MoveTo((self.x(), self.y()).into()),
                PathEl::LineTo((self.x(), self.y()).into()),
            ])
        }
    }

    impl IntoBezPathTolerance for &geo::MultiPoint<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            BezPath::from_vec(
                self.into_iter()
                    .flat_map(|pt| {
                        [
                            PathEl::MoveTo((pt.x(), pt.y()).into()),
                            PathEl::LineTo((pt.x(), pt.y()).into()),
                        ]
                    })
                    .collect(),
            )
        }
    }

    impl IntoBezPathTolerance for &geo::Line<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            BezPath::from_path_segments(std::iter::once(PathSeg::Line(kurbo::Line::new(
                (self.start.x, self.start.y),
                (self.end.x, self.end.y),
            ))))
        }
    }

    impl IntoBezPathTolerance for &geo::LineString<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            linestring_to_path_el(self).collect()
        }
    }

    impl IntoBezPathTolerance for &geo::MultiLineString<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            self.into_iter().flat_map(linestring_to_path_el).collect()
        }
    }

    impl IntoBezPathTolerance for &geo::Polygon<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            linestring_to_path_el(self.exterior())
                .chain(self.interiors().iter().flat_map(linestring_to_path_el))
                .collect()
        }
    }

    impl IntoBezPathTolerance for &geo::MultiPolygon<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            self.into_iter()
                .flat_map(|poly| {
                    linestring_to_path_el(poly.exterior())
                        .chain(poly.interiors().iter().flat_map(linestring_to_path_el))
                })
                .collect()
        }
    }

    impl IntoBezPathTolerance for &geo::GeometryCollection<f64> {
        fn into_bezpath_with_tolerance(self, tolerance: f64) -> BezPath {
            self.into_iter()
                .flat_map(|g| g.into_bezpath_with_tolerance(tolerance))
                .collect()
        }
    }

    impl IntoBezPathTolerance for &geo::Rect<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            BezPath::from_vec(vec![
                PathEl::MoveTo((self.min().x, self.min().y).into()),
                PathEl::LineTo((self.min().x, self.max().y).into()),
                PathEl::LineTo((self.max().x, self.max().y).into()),
                PathEl::LineTo((self.max().x, self.min().y).into()),
                PathEl::ClosePath,
            ])
        }
    }

    impl IntoBezPathTolerance for &geo::Triangle<f64> {
        fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
            BezPath::from_vec(vec![
                PathEl::MoveTo((self.0.x, self.0.y).into()),
                PathEl::LineTo((self.1.x, self.1.y).into()),
                PathEl::LineTo((self.2.x, self.2.y).into()),
                PathEl::ClosePath,
            ])
        }
    }

    impl IntoBezPathTolerance for &geo::Geometry<f64> {
        fn into_bezpath_with_tolerance(self, tolerance: f64) -> BezPath {
            match self {
                geo::Geometry::Point(point) => point.into_bezpath_with_tolerance(tolerance),
                geo::Geometry::Line(line) => line.into_bezpath_with_tolerance(tolerance),
                geo::Geometry::LineString(line_string) => {
                    line_string.into_bezpath_with_tolerance(tolerance)
                }
                geo::Geometry::Polygon(polygon) => polygon.into_bezpath_with_tolerance(tolerance),
                geo::Geometry::MultiPoint(multi_point) => {
                    multi_point.into_bezpath_with_tolerance(tolerance)
                }
                geo::Geometry::MultiLineString(multi_line_string) => {
                    multi_line_string.into_bezpath_with_tolerance(tolerance)
                }
                geo::Geometry::MultiPolygon(multi_polygon) => {
                    multi_polygon.into_bezpath_with_tolerance(tolerance)
                }
                geo::Geometry::GeometryCollection(geometry_collection) => {
                    geometry_collection.into_bezpath_with_tolerance(tolerance)
                }
                geo::Geometry::Rect(rect) => rect.into_bezpath_with_tolerance(tolerance),
                geo::Geometry::Triangle(triangle) => {
                    triangle.into_bezpath_with_tolerance(tolerance)
                }
            }
        }
    }

    // Macro to implement `IntoBezPathTolerance` for non-reference types (e.g., `geo::Polygon`),
    // by taking a reference and delegating to the implementations given above.
    macro_rules! geo_object_into_bezpath {
        ( $ t: ty) => {
            impl IntoBezPathTolerance for $t {
                #[inline]
                fn into_bezpath_with_tolerance(self, tolerance: f64) -> BezPath {
                    (&self).into_bezpath_with_tolerance(tolerance)
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
    geo_object_into_bezpath!(geo::Geometry<f64>);
    geo_object_into_bezpath!(geo::GeometryCollection<f64>);

    pub(super) fn linestring_to_path_el(
        ls: &geo::LineString<f64>,
    ) -> impl Iterator<Item = PathEl> + '_ {
        let closed = ls.is_closed();
        let len = ls.0.len();

        ls.into_iter().enumerate().map(move |(i, &pt)| {
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
    use super::*;
    use kurbo::PathEl;

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

    #[cfg(feature = "geo")]
    mod geo_tests {
        use super::super::geo_impl::*;
        use super::*;

        #[test]
        fn test_linestring_to_path_el() {
            // empty case
            assert_eq!(
                linestring_to_path_el(&geo::LineString(vec![])).collect::<Vec<_>>(),
                vec![]
            );

            // single point, should be just a single MoveTo
            assert_eq!(
                linestring_to_path_el(&geo::LineString(vec![(0., 0.).into()])).collect::<Vec<_>>(),
                vec![PathEl::MoveTo((0., 0.).into())]
            );

            // three points, not closed
            assert_eq!(
                linestring_to_path_el(&geo::LineString(vec![
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
                linestring_to_path_el(&geo::LineString(vec![
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
                linestring_to_path_el(&ls).collect::<Vec<_>>(),
                vec![
                    PathEl::MoveTo((0., 0.).into()),
                    PathEl::LineTo((1., 1.).into()),
                    PathEl::LineTo((2., 2.).into()),
                    PathEl::ClosePath,
                ]
            );
        }

        #[test]
        fn test_polygon_to_bezpath() {
            let poly = geo::Polygon::new(
                vec![
                    geo::Coord { x: 0.0, y: 0.0 },
                    geo::Coord { x: 1.0, y: 0.0 },
                    geo::Coord { x: 1.0, y: 1.0 },
                    geo::Coord { x: 0.0, y: 1.0 },
                ]
                .into(),
                vec![],
            );

            let bezpath = (&poly).into_bezpath();
            assert_eq!(
                bezpath,
                BezPath::from_vec(vec![
                    PathEl::MoveTo(kurbo::Point::new(0.0, 0.0)),
                    PathEl::LineTo(kurbo::Point::new(1.0, 0.0)),
                    PathEl::LineTo(kurbo::Point::new(1.0, 1.0)),
                    PathEl::LineTo(kurbo::Point::new(0.0, 1.0)),
                    PathEl::ClosePath,
                ])
            );

            // We can reuse `poly` since we only used a reference to it earlier.
            assert_eq!(poly.into_bezpath(), bezpath);
        }
    }
}
