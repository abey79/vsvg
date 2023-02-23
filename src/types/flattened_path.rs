use crate::types::path::PathImpl;

pub type Polyline = Vec<[f64; 2]>;

pub type FlattenedPath = PathImpl<Polyline>;

impl From<Polyline> for FlattenedPath {
    fn from(points: Polyline) -> Self {
        Self {
            data: points,
            ..Default::default()
        }
    }
}
