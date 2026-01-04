mod flattened_layer;
#[allow(clippy::module_inception)]
mod layer;
mod metadata;

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

    /// Join paths whose endpoints are within tolerance.
    ///
    /// If `flip` is true, paths may be reversed to enable more joins.
    ///
    /// Unlike [`sort`](LayerTrait::sort) which reorders paths, `join_paths` concatenates
    /// them, reducing the total path count.
    ///
    /// Note: Currently joins are only made at path endpoints. A future enhancement
    /// could re-loop closed paths when another path's endpoint touches any point
    /// along the closed path, enabling more joins.
    fn join_paths(&mut self, tolerance: f64, flip: bool) {
        if self.paths().len() <= 1 {
            return;
        }

        let taken_paths = std::mem::take(self.paths_mut());
        let mut index = IndexBuilder::default().flip(flip).build(&taken_paths);
        let mut result: Vec<P> = Vec::new();

        // Start first chain
        let Some(first_item) = index.pop_first() else {
            return;
        };
        let mut current = first_item.path.clone();

        // Greedy chain building
        loop {
            let Some(current_end) = current.end() else {
                result.push(current);
                match index.pop_first() {
                    Some(item) => current = item.path.clone(),
                    None => break,
                }
                continue;
            };

            // Find nearest path within tolerance
            if let Some((item, reversed)) = index.pop_nearest(&current_end) {
                let candidate_start = if reversed {
                    item.end.unwrap_or(current_end)
                } else {
                    item.start.unwrap_or(current_end)
                };

                if current_end.distance(&candidate_start) <= tolerance {
                    // Join this path
                    let mut next = item.path.clone();
                    if reversed {
                        next.data_mut().flip();
                    }
                    current.join(&next, tolerance);
                    // Continue trying to extend
                } else {
                    // Too far, start new chain
                    result.push(current);
                    current = item.path.clone();
                    if reversed {
                        current.data_mut().flip();
                    }
                }
            } else {
                // No more paths in index
                result.push(current);
                break;
            }
        }

        // Add remaining paths from index (shouldn't happen normally)
        while let Some(item) = index.pop_first() {
            result.push(item.path.clone());
        }

        *self.paths_mut() = result;
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
