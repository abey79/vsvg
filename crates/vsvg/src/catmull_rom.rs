use crate::{IntoBezPathTolerance, Point};
use kurbo::{BezPath, CubicBez};

/// A Catmull-Rom spline.
///
/// This is a curve that passes through all the points in the list, but is smooth. It is converted
/// to a series of cubic Bézier curves.
///
/// A minimum of 3 control points are needed to create a curve. If less than 3 points are provided,
/// an empty path is returned.
///
/// Strictly speaking, a Catmull-Rom spline starts at the second point and ends at the second to
/// last. This implementation duplicates the first and last points instead, so that all points are
/// traversed by the curve.
///
/// The Catmull-Rom spline is parameterized by a `tension` parameter. A tension of 1.0 is the
/// default. Higher values will make the curve more "tight" (closer to a polyline), while lower
/// values will make it more "loose". Values bellow ~0.2 yield very loose curves that can extend far
/// away from the control points. Negative values makes the curve pass backwards through control
/// points.
#[derive(Debug, Clone)]
pub struct CatmullRom {
    points: Vec<Point>,
    tension: f64,
}

impl Default for CatmullRom {
    fn default() -> Self {
        Self {
            tension: 1.0,
            points: Vec::new(),
        }
    }
}

impl CatmullRom {
    /// Create a new Catmull-Rom spline from a list of points.
    ///
    /// The `tension` parameter is set to 1.0.
    pub fn from_points(points: impl IntoIterator<Item = impl Into<Point>>) -> Self {
        Self {
            tension: 1.0,
            points: points.into_iter().map(Into::into).collect(),
        }
    }

    /// Set the tension parameter.
    #[must_use]
    pub fn tension(mut self, tension: f64) -> Self {
        self.tension = tension;
        self
    }

    /// Get the control points.
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Add a control point.
    pub fn push_point(&mut self, point: impl Into<Point>) {
        self.points.push(point.into());
    }
}

fn cubic_bez_from_catmull_rom_points(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    tension: f64,
) -> CubicBez {
    CubicBez::new(
        p1,
        p1 + (p2 - p0) / 6.0 / tension,
        p2 - (p3 - p1) / 6.0 / tension,
        p2,
    )
}

impl IntoBezPathTolerance for CatmullRom {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> BezPath {
        let tension = self.tension;

        if self.points.len() < 3 {
            return BezPath::new();
        }

        BezPath::from_path_segments(
            [cubic_bez_from_catmull_rom_points(
                self.points[0],
                self.points[0],
                self.points[1],
                self.points[2],
                tension,
            )]
            .into_iter()
            .chain(
                self.points
                    .windows(4)
                    .map(|w| cubic_bez_from_catmull_rom_points(w[0], w[1], w[2], w[3], tension)),
            )
            .chain([cubic_bez_from_catmull_rom_points(
                self.points[self.points.len() - 3],
                self.points[self.points.len() - 2],
                self.points[self.points.len() - 1],
                self.points[self.points.len() - 1],
                tension,
            )])
            .map(kurbo::PathSeg::Cubic),
        )
    }
}
