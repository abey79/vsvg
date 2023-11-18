mod flattened_layer;
#[allow(clippy::module_inception)]
mod layer;
mod metadata;

use crate::{IndexBuilder, PathDataTrait, PathTrait, Point, Transforms};

use crate::stats::LayerStats;
pub use flattened_layer::FlattenedLayer;
pub use layer::Layer;
pub use metadata::LayerMetadata;

pub trait LayerTrait<P: PathTrait<D>, D: PathDataTrait>: Default + Transforms {
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    #[must_use]
    fn from_paths_and_metadata(paths: Vec<P>, metadata: LayerMetadata) -> Self;

    fn paths(&self) -> &[P];

    fn paths_mut(&mut self) -> &mut Vec<P>;

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

    fn push_path(&mut self, path: impl Into<P>) {
        self.paths_mut().push(path.into());
    }

    /// Merge another layer into this one.
    ///
    /// Also merges the metadata, see [`LayerMetadata::merge`].
    fn merge(&mut self, other: &Self) {
        self.paths_mut().extend_from_slice(other.paths());
        self.metadata_mut().merge(other.metadata());
        //TODO(#4): merge default path metadata and cascade difference to paths
    }

    fn sort(&mut self, flip: bool) {
        self.sort_with_builder(IndexBuilder::default().flip(flip));
    }

    fn sort_with_builder(&mut self, builder: IndexBuilder) {
        if self.paths().len() <= 1 {
            return;
        }

        let mut new_paths = Vec::with_capacity(self.paths().len());
        let mut index = builder.build(self.paths());

        let mut pos = Point::ZERO;
        while let Some((path_item, reverse)) = index.pop_nearest(&pos) {
            new_paths.push((*path_item.path).clone());
            if reverse {
                pos = path_item.start.unwrap_or(pos);
                new_paths
                    .last_mut()
                    .expect("just inserted")
                    .data_mut()
                    .flip();
            } else {
                pos = path_item.end.unwrap_or(pos);
            }
        }

        // add any remaining, unindexed paths
        while let Some(path_item) = index.pop_first() {
            new_paths.push((*path_item.path).clone());
        }

        *self.paths_mut() = new_paths;
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
