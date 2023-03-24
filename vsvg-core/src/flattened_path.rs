use crate::path::PathImpl;
use crate::point::Point;
use crate::PathType;

pub type Polyline = Vec<Point>;

pub type FlattenedPath = PathImpl<Polyline>;

impl From<Polyline> for FlattenedPath {
    fn from(points: Polyline) -> Self {
        Self {
            data: points,
            ..Default::default()
        }
    }
}

impl PathType for Polyline {
    fn bounds(&self) -> kurbo::Rect {
        assert!(!self.is_empty(), "Cannot compute bounds of empty polyline");

        let rect = kurbo::Rect::from_center_size(self[0], (0.0, 0.0));
        self.iter()
            .skip(1)
            .fold(rect, |acc, point| acc.union_pt(point.into()))
    }

    fn start(&self) -> Option<Point> {
        self.first().copied()
    }

    fn end(&self) -> Option<Point> {
        self.last().copied()
    }

    fn is_point(&self) -> bool {
        self.len() == 1
    }

    fn flip(&mut self) {
        self.reverse();
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
        let points: Polyline = vec![];
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
