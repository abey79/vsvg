mod flattened_path;
mod metadata;
mod path;
mod point;

use crate::svg_writer::SvgPathWriter;
use crate::Transforms;

pub use flattened_path::{FlattenedPath, Polyline};
pub use metadata::PathMetadata;
pub use path::Path;
pub use point::Point;

pub trait PathDataTrait:
    Transforms + SvgPathWriter + Default + Clone + PartialEq + std::fmt::Debug
{
    fn bounds(&self) -> kurbo::Rect;
    fn start(&self) -> Option<Point>;
    fn end(&self) -> Option<Point>;
    fn is_point(&self) -> bool;
    fn flip(&mut self);
}

pub trait PathTrait<D: PathDataTrait>: Transforms {
    fn data(&self) -> &D;
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
