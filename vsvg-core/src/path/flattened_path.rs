use super::{PathDataTrait, PathMetadata, Point};
use crate::{PathTrait, Transforms};
use kurbo::Affine;

// ======================================================================================
// The path data for `FlattenedPath` is `Polyline`.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polyline(Vec<Point>);

impl Polyline {
    #[must_use]
    pub fn new(points: Vec<Point>) -> Self {
        Self(points)
    }

    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.0
    }

    #[must_use]
    pub fn points_mut(&mut self) -> &mut Vec<Point> {
        &mut self.0
    }
}

impl Transforms for Polyline {
    fn transform(&mut self, affine: &Affine) {
        for point in self.points_mut() {
            *point = *affine * *point;
        }
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
            .fold(rect, |acc, point| acc.union_pt(point.into()))
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
    metadata: PathMetadata,
}

impl Transforms for FlattenedPath {
    fn transform(&mut self, affine: &Affine) {
        self.data.transform(affine);
    }
}
impl PathTrait<Polyline> for FlattenedPath {
    fn data(&self) -> &Polyline {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Polyline {
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

    //TODO: test start, end, is_point
}
