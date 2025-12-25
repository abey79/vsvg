use std::ops::Range;

use super::{PathDataTrait, PathMetadata, Point};
use crate::{Path, PathTrait, Transforms};
use kurbo::Affine;

// ======================================================================================
// The path data for `FlattenedPath` is `Polyline`.

/// A [`Polyline`] is a sequence of connected [`Point`]s. It's considered closed if the first and
/// last points are the same.
///
/// [`Polyline`] is the data structure used by [`FlattenedPath`].
///
/// It can be constructed with a vector of [`Point`]s, or from an iterator of [`Point`]-compatible
/// items.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polyline(Vec<Point>);

impl Polyline {
    /// Crate a new [`Polyline`] from a vector of [`Point`].
    #[must_use]
    pub fn new(points: Vec<Point>) -> Self {
        Self(points)
    }

    /// Ensures the polyline is closed.
    pub fn close(&mut self) {
        if self.0.is_empty() || self.0[0] == self.0[self.0.len() - 1] {
            return;
        }
        self.0.push(self.0[0]);
    }

    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.0
    }

    #[must_use]
    pub fn points_mut(&mut self) -> &mut Vec<Point> {
        &mut self.0
    }

    #[must_use]
    pub fn into_points(self) -> Vec<Point> {
        self.0
    }
}

impl<P: Into<Point>> FromIterator<P> for Polyline {
    fn from_iter<T: IntoIterator<Item = P>>(points: T) -> Self {
        Self(points.into_iter().map(Into::into).collect())
    }
}

impl Transforms for Polyline {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        for point in self.points_mut() {
            *point = *affine * *point;
        }
        self
    }
}

impl PathDataTrait for Polyline {
    fn bounds(&self) -> kurbo::Rect {
        assert!(
            !self.0.is_empty(),
            "Cannot compute bounds of empty polyline"
        );

        let rect = kurbo::Rect::from_center_size(self.points()[0], (0.0, 0.0));
        self.points()
            .iter()
            .skip(1)
            .fold(rect, |acc, point| acc.union_pt(*point))
    }

    fn start(&self) -> Option<Point> {
        self.0.first().copied()
    }

    fn end(&self) -> Option<Point> {
        self.0.last().copied()
    }

    fn is_point(&self) -> bool {
        self.0.len() == 1
    }

    fn flip(&mut self) {
        self.0.reverse();
    }
}

// ======================================================================================
// `FlattenedPath`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlattenedPath {
    pub data: Polyline,
    pub(crate) metadata: PathMetadata,
}

impl Transforms for FlattenedPath {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        self.data.transform(affine);
        self
    }
}
impl PathTrait<Polyline> for FlattenedPath {
    fn data(&self) -> &Polyline {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Polyline {
        &mut self.data
    }

    fn into_data(self) -> Polyline {
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

impl From<Polyline> for FlattenedPath {
    fn from(points: Polyline) -> Self {
        Self {
            data: points,
            ..Default::default()
        }
    }
}

impl From<Vec<Point>> for FlattenedPath {
    fn from(points: Vec<Point>) -> Self {
        Polyline::new(points).into()
    }
}

// ======================================================================================
// Curve fitting support

/// Helper struct for implementing `kurbo::ParamCurveFit` on a polyline.
///
/// This enables using kurbo's `fit_to_bezpath` algorithm to fit smooth Bézier curves
/// to a sequence of points.
struct PolylineCurveFit<'a> {
    points: &'a [Point],
    cumulative_len: Vec<f64>,
    total_len: f64,
}

impl<'a> PolylineCurveFit<'a> {
    fn new(points: &'a [Point]) -> Self {
        // Store cumulative length at the END of each segment
        // So cumulative_len[i] = length from point 0 to point i+1
        let n_segments = points.len().saturating_sub(1);
        let mut cumulative_len = Vec::with_capacity(n_segments);
        let mut total = 0.0;

        for i in 0..n_segments {
            total += points[i + 1].distance(&points[i]);
            cumulative_len.push(total);
        }

        Self {
            points,
            cumulative_len,
            total_len: total,
        }
    }

    /// Map parameter t ∈ [0,1] to a point and tangent on the polyline.
    fn sample(&self, t: f64) -> (kurbo::Point, kurbo::Vec2) {
        if self.points.len() < 2 || self.total_len < 1e-10 {
            let p = self
                .points
                .first()
                .map_or(kurbo::Point::ZERO, |p| kurbo::Point::new(p.x(), p.y()));
            return (p, kurbo::Vec2::ZERO);
        }

        let target_len = t.clamp(0.0, 1.0) * self.total_len;

        // Binary search to find the segment containing target_len
        // cumulative_len[i] is the cumulative length at the end of segment i
        let seg_idx = self
            .cumulative_len
            .partition_point(|&l| l < target_len)
            .min(self.cumulative_len.len() - 1);

        let seg_start_len = if seg_idx == 0 {
            0.0
        } else {
            self.cumulative_len[seg_idx - 1]
        };
        let seg_end_len = self.cumulative_len[seg_idx];
        let seg_len = seg_end_len - seg_start_len;

        let local_t = if seg_len > 1e-10 {
            ((target_len - seg_start_len) / seg_len).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let p0 = &self.points[seg_idx];
        let p1 = &self.points[seg_idx + 1];

        let point = kurbo::Point::new(
            p0.x() + local_t * (p1.x() - p0.x()),
            p0.y() + local_t * (p1.y() - p0.y()),
        );
        let tangent = kurbo::Vec2::new(p1.x() - p0.x(), p1.y() - p0.y());

        (point, tangent)
    }
}

impl kurbo::ParamCurveFit for PolylineCurveFit<'_> {
    fn sample_pt_tangent(&self, t: f64, _sign: f64) -> kurbo::CurveFitSample {
        let (p, tangent) = self.sample(t);
        kurbo::CurveFitSample { p, tangent }
    }

    fn sample_pt_deriv(&self, t: f64) -> (kurbo::Point, kurbo::Vec2) {
        self.sample(t)
    }

    fn break_cusp(&self, _range: Range<f64>) -> Option<f64> {
        // Polylines are piecewise linear, no cusps
        None
    }
}

impl FlattenedPath {
    /// Fit smooth Bézier curves to this polyline.
    ///
    /// This is the inverse of [`Path::flatten`]: it converts a sequence of points back into
    /// smooth curves using kurbo's curve fitting algorithm.
    ///
    /// # Arguments
    ///
    /// * `tolerance` - Controls how closely the fitted curve follows the original points.
    ///   Smaller values produce tighter fits with more curve segments; larger values produce
    ///   smoother curves that may deviate more from the original points.
    ///
    /// # Returns
    ///
    /// A [`Path`] containing Bézier curves that approximate this polyline. The path's metadata
    /// is preserved from the original `FlattenedPath`.
    ///
    /// # Example
    ///
    /// ```
    /// use vsvg::{FlattenedPath, Point};
    ///
    /// let points = vec![
    ///     Point::new(0.0, 0.0),
    ///     Point::new(50.0, 100.0),
    ///     Point::new(100.0, 0.0),
    /// ];
    /// let flattened = FlattenedPath::from(points);
    /// let path = flattened.fit_to_path(1.0);
    /// ```
    #[must_use]
    pub fn fit_to_path(&self, tolerance: f64) -> Path {
        let points = self.data.points();

        if points.len() < 2 {
            // Not enough points to fit, return a simple path
            return Path::from_metadata(self.data.clone(), self.metadata.clone());
        }

        let curve_fit = PolylineCurveFit::new(points);
        let bezpath = kurbo::fit_to_bezpath(&curve_fit, tolerance);

        Path {
            data: bezpath,
            metadata: self.metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flattened_path_bounds() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 1.0),
            Point::new(2.0, 2.0),
        ];
        let path = FlattenedPath::from(points);
        assert_eq!(
            path.bounds(),
            kurbo::Rect::from_points((0.0, 0.0), (10.0, 2.0))
        );
    }

    #[test]
    #[should_panic]
    fn test_flattened_path_bounds_empty() {
        let points = Polyline::default();
        let path = FlattenedPath::from(points);
        path.bounds();
    }

    #[test]
    fn test_flattened_path_bounds_2_points() {
        let points = vec![Point::new(10.0, 0.0), Point::new(100.0, 13.0)];
        let path = FlattenedPath::from(points);
        assert_eq!(
            path.bounds(),
            kurbo::Rect::from_points((10.0, 0.0), (100.0, 13.0))
        );
    }

    #[test]
    fn test_polyline() {
        let mut p = Polyline::from_iter([(0., 0.), (1., 0.), (1., 1.)]);

        assert_eq!(
            p.points(),
            &[Point::new(0., 0.), Point::new(1., 0.), Point::new(1., 1.)]
        );

        assert_eq!(p.start(), Some(Point::new(0., 0.)));
        assert_eq!(p.end(), Some(Point::new(1., 1.)));
        assert!(!p.is_point());

        p.close();
        assert_eq!(
            p.points(),
            &[
                Point::new(0., 0.),
                Point::new(1., 0.),
                Point::new(1., 1.),
                Point::new(0., 0.)
            ]
        );
    }

    #[test]
    fn test_polyline_is_point() {
        let mut p = Polyline::from_iter([(0., 0.)]);

        assert_eq!(p.points(), &[Point::new(0., 0.)]);

        assert_eq!(p.start(), Some(Point::new(0., 0.)));
        assert_eq!(p.end(), Some(Point::new(0., 0.)));
        assert!(p.is_point());

        let p1 = p.clone();
        p.close();
        assert_eq!(p, p1);
    }
}
