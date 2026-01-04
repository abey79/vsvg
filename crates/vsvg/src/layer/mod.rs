mod flattened_layer;
#[allow(clippy::module_inception)]
mod layer;
mod metadata;

use crate::optimization;
use crate::path_index::IndexBuilder;
use crate::{PathDataTrait, PathTrait, Point, Transforms};

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

    fn for_each<F>(&mut self, f: F)
    where
        F: Fn(&mut P),
    {
        self.paths_mut().iter_mut().for_each(f);
    }

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

    /// Sort paths to minimize pen-up travel distance.
    ///
    /// See [`optimization::sort_paths`] for details.
    fn sort(&mut self, flip: bool) {
        optimization::sort_paths(self.paths_mut(), flip);
    }

    /// Sort paths with custom [`IndexBuilder`] settings.
    ///
    /// See [`optimization::sort_paths_with_builder`] for details.
    fn sort_with_builder(&mut self, builder: IndexBuilder) {
        optimization::sort_paths_with_builder(self.paths_mut(), builder);
    }

    /// Join paths whose endpoints are within tolerance.
    ///
    /// See [`optimization::join_paths`] for details.
    fn join_paths(&mut self, tolerance: f64, flip: bool) {
        optimization::join_paths(self.paths_mut(), tolerance, flip);
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

    /// Split all compound paths into individual subpaths.
    ///
    /// This is useful before [`Layer::join_paths`](crate::Layer::join_paths) to maximize
    /// optimization opportunities. When paths contain multiple subpaths (e.g.,
    /// from SVG imports or boolean operations), only the overall start/end
    /// points are considered for joining. Exploding first exposes all subpath
    /// endpoints.
    ///
    /// For [`FlattenedLayer`], this is a no-op since polylines cannot be compound.
    fn explode(&mut self) {
        let paths = std::mem::take(self.paths_mut());
        *self.paths_mut() = paths.into_iter().flat_map(P::split).collect();
    }
}
