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
