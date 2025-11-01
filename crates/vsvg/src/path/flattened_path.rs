use super::{PathDataTrait, PathMetadata, Point};
use crate::{PathTrait, Transforms};
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
            .fold(rect, |acc, point| acc.union_pt(point))
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
