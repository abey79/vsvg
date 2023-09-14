// inspiration: https://nb.paulbutler.org/optimizing-plots-with-tsp-solver/
// design note: https://github.com/abey79/vsvg/issues/12

use crate::{PathDataTrait, PathTrait, Point};
use bitvec::prelude::BitVec;
use indexmap::IndexMap;
use kdtree::distance::squared_euclidean;

type KdTree = kdtree::KdTree<f64, usize, [f64; 2]>;

pub struct PathIndex<'a, P: PathTrait<D>, D: PathDataTrait> {
    paths: IndexMap<usize, PathItem<'a, P, D>>,
    occupancy: BitVec,
    tree: KdTree,
    settings: IndexBuilder,
    reindex_agent: ReindexAgent,
    _phantom_data: std::marker::PhantomData<D>,
}

#[derive(Debug)]
pub struct PathItem<'a, P: PathTrait<D>, D: PathDataTrait> {
    pub path: &'a P,
    pub start: Option<Point>,
    pub end: Option<Point>,
    _phantom_data: std::marker::PhantomData<D>,
}

impl<'a, P: PathTrait<D>, D: PathDataTrait> From<&'a P> for PathItem<'a, P, D> {
    fn from(value: &'a P) -> Self {
        Self {
            path: value,
            start: value.start(),
            end: value.end(),
            _phantom_data: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ReindexStrategy {
    #[default]
    Default,
    Never,
    Threshold(usize),
    Ratio(f32),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IndexBuilder {
    pub flip: bool,
    pub reindex_strategy: ReindexStrategy,
    pub strict_order: bool,
}

impl IndexBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }

    #[must_use]
    pub fn strategy(mut self, reindex_strategy: ReindexStrategy) -> Self {
        self.reindex_strategy = reindex_strategy;
        self
    }

    #[must_use]
    pub fn strict_order(mut self, val: bool) -> Self {
        self.strict_order = val;
        self
    }

    #[must_use]
    pub fn build<P: PathTrait<D>, D: PathDataTrait>(self, paths: &[P]) -> PathIndex<P, D> {
        PathIndex::new(paths, self)
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, Default)]
struct ReindexAgent {
    strategy: ReindexStrategy,
    missed_accesses: usize,
    total_count: usize,
    threshold: usize,
}

/// This class implements the desired reindexing behaviour based on the strategy
impl ReindexAgent {
    const MIN_THRESHOLD: usize = 200;

    fn new(strategy: ReindexStrategy, total_count: usize) -> Self {
        Self {
            strategy,
            missed_accesses: 0,
            total_count,
            threshold: match strategy {
                // default is 40% of total count, see https://github.com/abey79/vsvg/issues/12
                ReindexStrategy::Default => (total_count * 2 / 5).max(Self::MIN_THRESHOLD),
                ReindexStrategy::Never => usize::MAX,
                ReindexStrategy::Threshold(t) => t,
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss
                )]
                ReindexStrategy::Ratio(f) => {
                    ((total_count as f32 * f.abs()) as usize).max(Self::MIN_THRESHOLD)
                }
            },
        }
    }

    #[inline]
    fn missed_access(&mut self) {
        self.missed_accesses += 1;
    }

    fn should_reindex(&mut self) -> bool {
        if self.missed_accesses >= self.threshold {
            self.missed_accesses = 0;
            true
        } else {
            false
        }
    }
}

impl<'a, P: PathTrait<D>, D: PathDataTrait> PathIndex<'a, P, D> {
    /// Create an index from a list of paths
    ///
    /// The order of the paths is reversed such as to have an efficient implementation of
    /// [`pop_first`] based on [`IndexMap::pop`].
    fn new(paths: &'a [P], settings: IndexBuilder) -> Self {
        let mut path_map = IndexMap::with_capacity(paths.len());
        for (idx, path) in paths.iter().rev().enumerate() {
            let path_item = path.into();
            path_map.insert(idx, path_item);
        }

        let mut pi = Self {
            paths: path_map,
            occupancy: BitVec::repeat(true, paths.len()),
            tree: KdTree::new(2),
            settings,
            reindex_agent: ReindexAgent::default(), // will be overwritten in reindex
            _phantom_data: std::marker::PhantomData,
        };

        pi.reindex();

        pi
    }

    fn reindex(&mut self) {
        // update reindex agent
        let total_count = if self.settings.flip {
            self.paths.len() * 2
        } else {
            self.paths.len()
        };
        self.reindex_agent = ReindexAgent::new(self.settings.reindex_strategy, total_count);

        // update k-d tree
        self.tree = KdTree::new(2);
        for (idx, path_item) in self.paths.iter() {
            if let Some(ref start) = path_item.start {
                if self.settings.flip {
                    if let Some(ref end) = path_item.end {
                        self.tree.add(start.into(), 2 * idx).unwrap();
                        self.tree.add(end.into(), 2 * idx + 1).unwrap();
                    }
                } else {
                    self.tree.add(start.into(), *idx).unwrap();
                }
            }
        }
    }

    pub fn pop_first(&mut self) -> Option<PathItem<P, D>> {
        // since the paths were reversed upon insertion, the pop operation corresponds to pop_first
        let (idx, path_item) = self.paths.pop()?;
        self.occupancy.set(idx, false);
        Some(path_item)
    }

    #[must_use]
    #[inline]
    fn tree_to_map_index(&self, tree_idx: usize) -> (usize, bool) {
        if self.settings.flip {
            (tree_idx / 2, tree_idx % 2 == 1)
        } else {
            (tree_idx, false)
        }
    }

    /// Return the nearest path to the given point.
    ///
    /// This function may return `None` even if the `PathIndex` is not empty, as some paths may not
    /// be indexed.
    pub fn pop_nearest(&mut self, point: &Point) -> Option<(PathItem<P, D>, bool)> {
        if self.reindex_agent.should_reindex() {
            self.reindex();
        }

        if self.tree.size() == 0 {
            return None;
        }

        let pt: [f64; 2] = point.into();
        // FIXME: error handling is not optimal here
        let iter = self.tree.iter_nearest(&pt, &squared_euclidean).ok()?;

        for (_, &tree_idx) in iter {
            let (idx, reversed) = self.tree_to_map_index(tree_idx);

            if self.occupancy[idx] {
                let path_item = if self.settings.strict_order {
                    self.paths.shift_remove(&idx)
                } else {
                    self.paths.swap_remove(&idx)
                };
                let path_item = path_item.expect("path cannot be in tree but not in map");

                self.occupancy.set(idx, false);
                return Some((path_item, reversed));
            }

            // register missed access
            self.reindex_agent.missed_access();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlattenedPath;

    #[test]
    fn test_path_index() {
        let paths = vec![
            FlattenedPath::from(vec![
                Point::new(0.0, 0.0),
                Point::new(13.0, 1.0),
                Point::new(1.0, 2.0),
            ]),
            FlattenedPath::from(vec![
                Point::new(5.0, 0.0),
                Point::new(11.0, 1.0),
                Point::new(6.0, 2.0),
            ]),
            FlattenedPath::from(vec![
                Point::new(1.0, 0.0),
                Point::new(10.0, 1.0),
                Point::new(2.0, 2.0),
            ]),
        ];
        let mut pi = IndexBuilder::new().build(&paths);
        assert_eq!(pi.pop_first().unwrap().path, &paths[0]);
        assert_eq!(pi.pop_first().unwrap().path, &paths[1]);
        assert_eq!(pi.pop_first().unwrap().path, &paths[2]);
        assert!(pi.pop_first().is_none());
    }

    fn assert_nearest<P: PathTrait<D>, D: PathDataTrait>(
        res: Option<(PathItem<P, D>, bool)>,
        expected_path: &P,
        expected_reversed: bool,
    ) {
        let (path_item, reversed) = res.expect("must find a path");
        assert_eq!(path_item.path, expected_path);
        assert_eq!(reversed, expected_reversed);
    }

    #[test]
    fn test_path_index_pop_nearest() {
        let paths = vec![
            FlattenedPath::from(vec![Point::new(1.0, 0.0), Point::new(2.0, 2.0)]),
            FlattenedPath::from(vec![Point::new(0.0, 0.5), Point::new(1.0, 2.0)]),
            FlattenedPath::from(vec![Point::new(5.0, 0.0), Point::new(6.0, 2.0)]),
        ];
        let mut pi = IndexBuilder::new().build(&paths);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[1], false);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[0], false);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[2], false);
        assert!(pi.pop_nearest(&Point::new(0.0, 0.0)).is_none());
        assert!(pi.pop_first().is_none());
    }

    #[test]
    fn test_path_index_pop_nearest_bidir() {
        let paths = vec![
            FlattenedPath::from(vec![Point::new(1.0, 0.0), Point::new(2.0, 2.0)]),
            FlattenedPath::from(vec![Point::new(0.0, 0.5), Point::new(1.0, 2.0)]),
            FlattenedPath::from(vec![Point::new(15.0, 0.0), Point::new(6.0, 2.0)]),
        ];
        let mut pi = IndexBuilder::new().flip(true).build(&paths);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[1], false);
        assert_nearest(pi.pop_nearest(&Point::new(2.0, 2.1)), &paths[0], true);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[2], true);
        assert!(pi.pop_nearest(&Point::new(0.0, 0.0)).is_none());
        assert!(pi.pop_first().is_none());
    }
}
