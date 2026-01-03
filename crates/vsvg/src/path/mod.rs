mod flattened_path;
pub(crate) mod into_bezpath;
mod metadata;
#[allow(clippy::module_inception)]
mod path;
mod point;

use crate::{SvgPathWriter, Transforms};

pub use flattened_path::{FlattenedPath, Polyline};
pub use into_bezpath::{IntoBezPath, IntoBezPathTolerance};
pub use metadata::PathMetadata;
pub use path::Path;
pub use point::Point;

pub const DEFAULT_TOLERANCE: f64 = 0.05;

/// Epsilon for considering two points as coincident.
pub const EPSILON: f64 = 1e-10;

pub trait PathDataTrait:
    Transforms + SvgPathWriter + Default + Clone + PartialEq + std::fmt::Debug
{
    fn bounds(&self) -> kurbo::Rect;
    fn start(&self) -> Option<Point>;
    fn end(&self) -> Option<Point>;
    fn is_point(&self) -> bool;
    fn flip(&mut self);

    /// Returns true if the path is closed (start ≈ end within [`EPSILON`]).
    fn is_closed(&self) -> bool {
        match (self.start(), self.end()) {
            (Some(start), Some(end)) => start.distance(&end) < EPSILON,
            _ => false,
        }
    }
}

pub trait PathTrait<D: PathDataTrait>: Transforms + Clone + PartialEq + std::fmt::Debug {
    fn data(&self) -> &D;

    fn data_mut(&mut self) -> &mut D;

    fn into_data(self) -> D;

    fn bounds(&self) -> kurbo::Rect;

    fn start(&self) -> Option<Point> {
        self.data().start()
    }

    fn end(&self) -> Option<Point> {
        self.data().end()
    }

    fn metadata(&self) -> &PathMetadata;
    fn metadata_mut(&mut self) -> &mut PathMetadata;
}
