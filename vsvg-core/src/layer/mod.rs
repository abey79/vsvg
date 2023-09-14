mod flattened_layer;
mod layer;
mod metadata;

use crate::{PathDataTrait, PathTrait, Point, Transforms};

use crate::stats::LayerStats;
pub use flattened_layer::FlattenedLayer;
pub use layer::Layer;
pub use metadata::LayerMetadata;

pub trait LayerTrait<P: PathTrait<D>, D: PathDataTrait>: Default + Transforms {
    fn paths(&self) -> &Vec<P>;

    fn metadata(&self) -> &LayerMetadata;

    fn metadata_mut(&mut self) -> &mut LayerMetadata;

    fn bounds(&self) -> Option<kurbo::Rect> {
        if self.paths().is_empty() {
            return None;
        }

        let first = self.paths().first().expect("checked").bounds();
        Some(
            self.paths()
                .iter()
                .skip(1)
                .fold(first, |acc, path| acc.union(path.bounds())),
        )
    }

    fn pen_up_trajectories(&self) -> Vec<(Point, Point)> {
        self.paths()
            .windows(2)
            .filter_map(|w| {
                if let Some(ref start) = w[0].end() {
                    #[allow(clippy::manual_map)]
                    if let Some(ref end) = w[1].start() {
                        Some((*start, *end))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn stats(&self) -> LayerStats {
        LayerStats::from_layer(self)
    }
}
