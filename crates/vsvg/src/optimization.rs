use crate::path_index::IndexBuilder;
use crate::{PathDataTrait, PathTrait, Point};

/// Sort paths to minimize pen-up travel distance.
///
/// Uses a greedy algorithm starting from the origin. Paths that cannot be
/// spatially indexed (empty or degenerate) are moved to the end.
///
/// If `flip` is true, paths may be reversed to enable shorter travel.
pub fn sort_paths<P, D>(paths: &mut Vec<P>, flip: bool)
where
    P: PathTrait<D>,
    D: PathDataTrait,
{
    sort_paths_with_builder(paths, IndexBuilder::default().flip(flip));
}

/// Sort paths with custom [`IndexBuilder`] settings.
#[allow(clippy::missing_panics_doc)]
pub fn sort_paths_with_builder<P, D>(paths: &mut Vec<P>, builder: IndexBuilder)
where
    P: PathTrait<D>,
    D: PathDataTrait,
{
    if paths.len() <= 1 {
        return;
    }

    let mut new_paths = Vec::with_capacity(paths.len());
    let mut index = builder.build(paths);

    let mut pos = Point::ZERO;
    while let Some((path_item, reverse)) = index.pop_nearest(&pos) {
        new_paths.push(path_item.path.clone());
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

    // Add any remaining, unindexed paths
    while let Some(path_item) = index.pop_first() {
        new_paths.push(path_item.path.clone());
    }

    *paths = new_paths;
}

/// Join paths whose endpoints are within tolerance.
///
/// Uses greedy chain building: repeatedly extend the current path by joining
/// the nearest path whose endpoint is within tolerance.
///
/// If `flip` is true, paths may be reversed to enable more joins.
///
/// Unlike [`sort_paths`] which reorders paths, `join_paths` concatenates
/// them, reducing the total path count.
pub fn join_paths<P, D>(paths: &mut Vec<P>, tolerance: f64, flip: bool)
where
    P: PathTrait<D>,
    D: PathDataTrait,
{
    if paths.len() <= 1 {
        return;
    }

    let taken_paths = std::mem::take(paths);
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

    *paths = result;
}

#[cfg(test)]
mod tests {
    use crate::{
        Document, DocumentTrait, FlattenedLayer, FlattenedPath, Layer, LayerTrait, Path,
        PathDataTrait, Point, test_file,
    };

    #[test]
    fn test_sort() {
        let mut layer = FlattenedLayer::default();

        let p1 = FlattenedPath::from(vec![Point::new(10.0, 10.1), Point::new(0.0, 0.0)]);
        let p2 = FlattenedPath::from(vec![Point::new(3.0, 2.3), Point::new(10.0, 10.0)]);
        let p3 = FlattenedPath::from(vec![Point::new(1.0, 0.0), Point::new(0.0, 0.0)]);
        let p4 = FlattenedPath::from(vec![Point::new(2.0, 1.0), Point::new(1.0, 0.1)]);

        layer.paths.push(p1.clone());
        layer.paths.push(p2.clone());
        layer.paths.push(p3.clone());
        layer.paths.push(p4.clone());

        layer.sort(false);

        assert_eq!(layer.paths[0], p3);
        assert_eq!(layer.paths[1], p4);
        assert_eq!(layer.paths[2], p2);
        assert_eq!(layer.paths[3], p1);
    }

    #[test]
    fn test_sort_bidir() {
        let mut layer = FlattenedLayer::default();

        let p1 = FlattenedPath::from(vec![Point::new(10.0, 10.1), Point::new(20.0, 20.0)]);
        let p2 = FlattenedPath::from(vec![Point::new(3.0, 2.3), Point::new(10.0, 10.0)]);
        let mut p3 = FlattenedPath::from(vec![Point::new(1.0, 0.0), Point::new(0.0, 0.0)]);
        let mut p4 = FlattenedPath::from(vec![Point::new(3.0, 2.0), Point::new(1.0, 0.1)]);

        layer.paths.push(p1.clone());
        layer.paths.push(p2.clone());
        layer.paths.push(p3.clone());
        layer.paths.push(p4.clone());

        layer.sort(true);

        p3.data.flip();
        assert_eq!(layer.paths[0], p3);
        p4.data.flip();
        assert_eq!(layer.paths[1], p4);
        assert_eq!(layer.paths[2], p2);
        assert_eq!(layer.paths[3], p1);
    }

    #[test]
    fn test_sort_problematic_case() {
        let mut doc = Document::from_svg(test_file!("random_100_sort.svg"), false).unwrap();
        doc.get_mut(1).sort(true);
    }

    // ==================== join_paths tests ====================

    #[test]
    fn test_join_paths_basic() {
        let mut layer = FlattenedLayer::default();

        // Two paths that should join: (0,0)->(10,0) and (10,0)->(20,0)
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        ]));
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(10.0, 0.0),
            Point::new(20.0, 0.0),
        ]));

        layer.join_paths(0.1, false);

        assert_eq!(layer.paths.len(), 1);
        // Should have 3 points (duplicate point at junction is skipped)
        assert_eq!(layer.paths[0].data.points().len(), 3);
    }

    #[test]
    fn test_join_paths_chain_of_three() {
        let mut layer = FlattenedLayer::default();

        // A: (0,0) -> (10,0)
        // B: (10,0) -> (10,10)
        // C: (10,10) -> (0,10)
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        ]));
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        ]));
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]));

        layer.join_paths(0.1, false);

        assert_eq!(layer.paths.len(), 1);
        assert_eq!(layer.paths[0].data.points().len(), 4);
    }

    #[test]
    fn test_join_paths_with_flip() {
        let mut layer = FlattenedLayer::default();

        // A: (0,0) -> (10,0)
        // B: (20,0) -> (10,0)  -- end of B matches end of A, needs flip
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        ]));
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(20.0, 0.0),
            Point::new(10.0, 0.0),
        ]));

        // Without flip: should not join (paths are 2)
        let mut layer_no_flip = layer.clone();
        layer_no_flip.join_paths(0.1, false);
        assert_eq!(layer_no_flip.paths.len(), 2);

        // With flip: should join into 1 path
        layer.join_paths(0.1, true);
        assert_eq!(layer.paths.len(), 1);
    }

    #[test]
    fn test_join_paths_no_join_too_far() {
        let mut layer = FlattenedLayer::default();

        layer.paths.push(FlattenedPath::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        ]));
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(15.0, 0.0), // 5 units away
            Point::new(25.0, 0.0),
        ]));

        layer.join_paths(1.0, false); // Tolerance 1.0

        assert_eq!(layer.paths.len(), 2); // No join
    }

    #[test]
    fn test_join_paths_closed_path_included() {
        let mut layer = FlattenedLayer::default();

        // Open path ending at (10,0)
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        ]));

        // Closed path (square) starting/ending at (10,0)
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0), // Closed: end == start
        ]));

        layer.join_paths(0.1, true);

        // Closed paths CAN be joined - open path's end meets closed path's start
        assert_eq!(layer.paths.len(), 1);
        // Open path (2 pts) + closed path (5 pts) - 1 duplicate = 6 pts
        assert_eq!(layer.paths[0].data.points().len(), 6);
    }

    #[test]
    fn test_join_paths_tolerance_boundary() {
        let mut layer = FlattenedLayer::default();

        layer.paths.push(FlattenedPath::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        ]));
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(11.0, 0.0), // Exactly 1.0 away
            Point::new(20.0, 0.0),
        ]));

        // At tolerance: should join
        layer.join_paths(1.0, false);
        assert_eq!(layer.paths.len(), 1);
    }

    #[test]
    fn test_join_paths_empty_layer() {
        let mut layer = FlattenedLayer::default();
        layer.join_paths(1.0, true);
        assert_eq!(layer.paths.len(), 0);
    }

    #[test]
    fn test_join_paths_single_path() {
        let mut layer = FlattenedLayer::default();
        layer.paths.push(FlattenedPath::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
        ]));

        layer.join_paths(1.0, true);
        assert_eq!(layer.paths.len(), 1);
    }

    #[test]
    fn test_join_paths_bezpath_layer() {
        // Test join_paths with Layer (BezPath-based) to ensure trait method works
        let mut layer = Layer::default();

        // Two paths that should join: (0,0)->(10,0) and (10,0)->(20,0)
        layer.paths.push(Path::from_svg("M 0,0 L 10,0").unwrap());
        layer.paths.push(Path::from_svg("M 10,0 L 20,0").unwrap());

        layer.join_paths(0.1, false);

        assert_eq!(layer.paths.len(), 1);
        // BezPath join converts MoveTo to LineTo: M 0,0 L 10,0 + L 10,0 L 20,0 = 4 elements
        assert_eq!(layer.paths[0].data.elements().len(), 4);
    }

    // ==================== explode tests ====================

    #[test]
    fn test_explode_simple_paths() {
        let mut layer = Layer::default();

        // Add two simple (non-compound) paths
        layer.paths.push(Path::from_svg("M 0,0 L 10,10").unwrap());
        layer.paths.push(Path::from_svg("M 20,20 L 30,30").unwrap());

        layer.explode();

        // Should still be 2 paths (no change)
        assert_eq!(layer.paths.len(), 2);
    }

    #[test]
    fn test_explode_compound_path() {
        let mut layer = Layer::default();

        // Add one compound path with 2 subpaths
        layer
            .paths
            .push(Path::from_svg("M 0,0 L 10,10 M 50,50 L 60,60").unwrap());

        layer.explode();

        // Should now be 2 separate paths
        assert_eq!(layer.paths.len(), 2);
        assert_eq!(layer.paths[0].data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(layer.paths[1].data.start(), Some(Point::new(50.0, 50.0)));
    }

    #[test]
    fn test_explode_mixed() {
        let mut layer = Layer::default();

        // Simple path
        layer.paths.push(Path::from_svg("M 0,0 L 10,10").unwrap());
        // Compound path with 3 subpaths
        layer
            .paths
            .push(Path::from_svg("M 20,20 L 30,30 M 40,40 L 50,50 M 60,60 L 70,70").unwrap());
        // Another simple path
        layer.paths.push(Path::from_svg("M 80,80 L 90,90").unwrap());

        layer.explode();

        // 1 + 3 + 1 = 5 paths
        assert_eq!(layer.paths.len(), 5);
    }

    #[test]
    fn test_explode_empty_layer() {
        let mut layer = Layer::default();
        layer.explode();
        assert_eq!(layer.paths.len(), 0);
    }

    #[test]
    fn test_explode_then_join() {
        let mut layer = Layer::default();

        // Compound path where subpaths are far apart
        // Subpath 1: (0,0) -> (10,0)
        // Subpath 2: (10,0) -> (20,0)  -- starts where subpath 1 ends!
        layer
            .paths
            .push(Path::from_svg("M 0,0 L 10,0 M 10,0 L 20,0").unwrap());

        // Before explode: join_paths won't join internal endpoints
        // The compound path has start=(0,0) and end=(20,0)

        // After explode: we have 2 paths that can be joined
        layer.explode();
        assert_eq!(layer.paths.len(), 2);

        layer.join_paths(0.1, false);
        assert_eq!(layer.paths.len(), 1);
    }
}
