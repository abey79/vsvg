// inspiration: https://nb.paulbutler.org/optimizing-plots-with-tsp-solver/

use crate::point::Point;
use crate::{PathImpl, PathType};
use indexmap::IndexMap;
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;

pub struct PathIndex<'a, T: PathType> {
    paths: IndexMap<usize, PathItem<'a, T>>,
    tree: KdTree<f64, 2>,
    bidirectional: bool,
}

pub struct PathItem<'a, T: PathType> {
    pub path: &'a PathImpl<T>,
    pub start: Option<Point>,
    pub end: Option<Point>,
}

impl<'a, T: PathType> From<&'a PathImpl<T>> for PathItem<'a, T> {
    fn from(value: &'a PathImpl<T>) -> Self {
        Self {
            path: value,
            start: value.data.start(),
            end: value.data.end(),
        }
    }
}

fn tree_add(tree: &mut KdTree<f64, 2>, point: &[f64; 2], idx: usize) {
    println!("add {:>7}: {:>10.6} {:>10.6}", idx, point[0], point[1]);
    tree.add(point, idx);
}

fn tree_remove(tree: &mut KdTree<f64, 2>, point: &[f64; 2], idx: usize) {
    let res = tree.remove(point, idx);
    println!(
        "rem {:>7}: {:>10.6} {:>10.6} {:>10}",
        idx, point[0], point[1], res
    );
}

impl<'a, T: PathType> PathIndex<'a, T> {
    /// Create an index from a list of paths
    ///
    /// The order of the paths is reversed such as to have an efficient implementation of
    /// [`pop_first`] based on [`IndexMap::pop`].
    pub fn new(paths: &'a [PathImpl<T>], bidirectional: bool) -> Self {
        let mut pi = Self {
            paths: IndexMap::with_capacity(paths.len()),
            tree: KdTree::with_capacity(
                if bidirectional {
                    paths.len() * 2
                } else {
                    paths.len()
                }
                .max(1),
            ),
            bidirectional,
        };

        for (idx, path) in paths.iter().rev().enumerate() {
            let path_item = path.into();
            pi.tree_operation(&path_item, idx, tree_add);
            pi.paths.insert(idx, path_item);
        }

        pi
    }

    /// Execute an operation of the tree for the given path item.
    ///
    /// This function captures the logic whether a given item is stored in the spatial index or not,
    /// which happens only if either of these conditions are true:
    ///
    /// - the index is not bidirectional and the item as a start point
    /// - the index is bidirectional and the item has both a start and an end point
    ///
    /// For the latter, the operation is executed twice, once for the start point and once for the
    /// end point, with the corresponding indices.
    fn tree_operation<Ignored>(
        &mut self,
        path_item: &PathItem<'a, T>,
        idx: usize,
        // the signature uses &[f64; 2] for compatibility with KdTree functions
        op: impl Fn(&mut KdTree<f64, 2>, &[f64; 2], usize) -> Ignored,
    ) {
        if let Some(ref start) = path_item.start {
            if self.bidirectional {
                if let Some(ref end) = path_item.end {
                    op(&mut self.tree, &start.into(), 2 * idx);
                    op(&mut self.tree, &end.into(), 2 * idx + 1);
                }
            } else {
                op(&mut self.tree, &start.into(), idx);
            }
        }
    }

    pub fn pop_first(&mut self) -> Option<PathItem<T>> {
        // since the paths were reversed upon insertion, the pop operation corresponds to pop_first
        let (idx, path_item) = self.paths.pop()?;
        self.tree_operation(&path_item, idx, tree_remove);
        Some(path_item)
    }

    #[must_use]
    #[inline]
    fn tree_to_map_index(&self, tree_idx: usize) -> (usize, bool) {
        if self.bidirectional {
            (tree_idx / 2, tree_idx % 2 == 1)
        } else {
            (tree_idx, false)
        }
    }

    /// Return the nearest path to the given point.
    ///
    /// This function may return `None` even if the `PathIndex` is not empty, as some paths may not
    /// be indexed.
    pub fn pop_nearest(&mut self, point: &Point) -> Option<(PathItem<T>, bool)> {
        if self.tree.size() == 0 {
            return None;
        }

        let (_, tree_idx) = self.tree.nearest_one(&point.into(), &squared_euclidean);
        let (idx, reversed) = self.tree_to_map_index(tree_idx);

        println!("fnd {tree_idx:>7}: {idx:>10} {reversed:>10}");

        let path_item = self
            .paths
            .shift_remove(&idx)
            .expect("path cannot be in tree but not in map");
        self.tree_operation(&path_item, idx, tree_remove);

        Some((path_item, reversed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_index() {
        let paths = vec![
            PathImpl::from(vec![
                Point::new(0.0, 0.0),
                Point::new(13.0, 1.0),
                Point::new(1.0, 2.0),
            ]),
            PathImpl::from(vec![
                Point::new(5.0, 0.0),
                Point::new(11.0, 1.0),
                Point::new(6.0, 2.0),
            ]),
            PathImpl::from(vec![
                Point::new(1.0, 0.0),
                Point::new(10.0, 1.0),
                Point::new(2.0, 2.0),
            ]),
        ];
        let mut pi = PathIndex::new(&paths, false);
        assert_eq!(pi.pop_first().unwrap().path, &paths[0]);
        assert_eq!(pi.pop_first().unwrap().path, &paths[1]);
        assert_eq!(pi.pop_first().unwrap().path, &paths[2]);
        assert!(pi.pop_first().is_none());
    }

    fn assert_nearest<T: PathType>(
        res: Option<(PathItem<T>, bool)>,
        expected_path: &PathImpl<T>,
        expected_reversed: bool,
    ) {
        let (path_item, reversed) = res.expect("must find a path");
        assert_eq!(path_item.path, expected_path);
        assert_eq!(reversed, expected_reversed);
    }

    #[test]
    fn test_path_index_pop_nearest() {
        let paths = vec![
            PathImpl::from(vec![Point::new(1.0, 0.0), Point::new(2.0, 2.0)]),
            PathImpl::from(vec![Point::new(0.0, 0.5), Point::new(1.0, 2.0)]),
            PathImpl::from(vec![Point::new(5.0, 0.0), Point::new(6.0, 2.0)]),
        ];
        let mut pi = PathIndex::new(&paths, false);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[1], false);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[0], false);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[2], false);
        assert!(pi.pop_nearest(&Point::new(0.0, 0.0)).is_none());
        assert!(pi.pop_first().is_none());
    }

    #[test]
    fn test_path_index_pop_nearest_bidir() {
        let paths = vec![
            PathImpl::from(vec![Point::new(1.0, 0.0), Point::new(2.0, 2.0)]),
            PathImpl::from(vec![Point::new(0.0, 0.5), Point::new(1.0, 2.0)]),
            PathImpl::from(vec![Point::new(15.0, 0.0), Point::new(6.0, 2.0)]),
        ];
        let mut pi = PathIndex::new(&paths, true);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[1], false);
        assert_nearest(pi.pop_nearest(&Point::new(2.0, 2.1)), &paths[0], true);
        assert_nearest(pi.pop_nearest(&Point::new(0.0, 0.0)), &paths[2], true);
        assert!(pi.pop_nearest(&Point::new(0.0, 0.0)).is_none());
        assert!(pi.pop_first().is_none());
    }

    #[ignore] // this test fails with kiddo 2.0.0-beta.5
    #[test]
    fn test_kiddo_bug() {
        let pts = vec![
            [19.2023, 7.1812],
            [7.6427, 22.5779],
            [26.6314, 34.8920],
            [36.7890, 27.2663],
            [28.3226, 8.5047],
            [5.3914, 28.1360],
            [5.0978, 3.6814],
            [0.5114, 11.6552],
            [4.7981, 21.6210],
            [29.0030, 9.6799],
            [35.5580, 1.8891],
            [3.9160, 25.5702],
            [22.2497, 31.6140],
            [30.7110, 36.7514],
            [0.2828, 12.4298],
            [20.0206, 3.0635],
            [30.6153, 2.8582],
            [23.7179, 6.2048],
            [13.0438, 4.2319],
            [4.6433, 30.9660],
            [5.0588, 5.2028],
            [19.2023, 23.7406],
            [37.3171, 32.7523],
            [12.6957, 15.7080],
            [15.6001, 14.3995],
            [36.0203, 21.0366],
            [6.3956, 2.7644],
            [3.1719, 8.7039],
            [0.9159, 12.2299],
            [23.8157, 14.0699],
            [27.7757, 7.3597],
            [28.4198, 31.3427],
            [2.3290, 6.2364],
            [10.1126, 7.7009],
        ];

        let mut tree = KdTree::<f64, 2>::new();

        for (i, pt) in pts.iter().enumerate() {
            tree.add(pt, i);
        }

        assert_eq!(tree.remove(&pts[0], 0), 1);
    }
}
