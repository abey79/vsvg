use crate::point::Point;
use crate::spatial_index::{PathIndex, PathIndexSettings, ReindexStrategy};
use crate::{LayerImpl, PathType};

impl<T: PathType> LayerImpl<T> {
    /// Sort the paths such as to minimize the pen up distance
    ///
    /// This is done using a greedy algorithm, starting with the layer's first path. Any path that
    /// cannot be spatially indexed (empty or otherwise degenerate) is moved at the end.
    pub fn sort(&mut self, flip: bool, reindex_threshold: usize) {
        if self.paths.len() <= 1 {
            return;
        }

        let mut new_paths = Vec::with_capacity(self.paths.len());
        let mut index = PathIndex::new(
            &self.paths,
            PathIndexSettings::default()
                .flip(flip)
                .strategy(ReindexStrategy::Threshold(reindex_threshold)),
        );

        let mut pos = Point::ZERO;
        let mut cnt = 0;
        while let Some((path_item, reverse)) = index.pop_nearest(&pos) {
            new_paths.push((*path_item.path).clone());
            if reverse {
                pos = path_item.start.unwrap_or(pos);
                new_paths.last_mut().expect("just inserted").data.flip();
            } else {
                pos = path_item.end.unwrap_or(pos);
            }
            cnt += 1;

            if cnt % 100 == 0 {
                println!("{} / {}", cnt, self.paths.len());
            }
        }

        // add any remaining, unindexed paths
        while let Some(path_item) = index.pop_first() {
            new_paths.push((*path_item.path).clone());
        }

        self.paths = new_paths;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flattened_layer::FlattenedLayer;
    use crate::point::Point;
    use crate::{test_file, Document, FlattenedPath};

    #[test]
    fn test_sort() {
        let mut layer = FlattenedLayer::new();

        let p1 = FlattenedPath::from(vec![Point::new(10.0, 10.1), Point::new(0.0, 0.0)]);
        let p2 = FlattenedPath::from(vec![Point::new(3.0, 2.3), Point::new(10.0, 10.0)]);
        let p3 = FlattenedPath::from(vec![Point::new(1.0, 0.0), Point::new(0.0, 0.0)]);
        let p4 = FlattenedPath::from(vec![Point::new(2.0, 1.0), Point::new(1.0, 0.1)]);

        layer.paths.push(p1.clone());
        layer.paths.push(p2.clone());
        layer.paths.push(p3.clone());
        layer.paths.push(p4.clone());

        layer.sort(false, 1000);

        assert_eq!(layer.paths[0], p3);
        assert_eq!(layer.paths[1], p4);
        assert_eq!(layer.paths[2], p2);
        assert_eq!(layer.paths[3], p1);
    }

    #[test]
    fn test_sort_bidir() {
        let mut layer = FlattenedLayer::new();

        let p1 = FlattenedPath::from(vec![Point::new(10.0, 10.1), Point::new(20.0, 20.0)]);
        let p2 = FlattenedPath::from(vec![Point::new(3.0, 2.3), Point::new(10.0, 10.0)]);
        let mut p3 = FlattenedPath::from(vec![Point::new(1.0, 0.0), Point::new(0.0, 0.0)]);
        let mut p4 = FlattenedPath::from(vec![Point::new(3.0, 2.0), Point::new(1.0, 0.1)]);

        layer.paths.push(p1.clone());
        layer.paths.push(p2.clone());
        layer.paths.push(p3.clone());
        layer.paths.push(p4.clone());

        layer.sort(true, 1000);

        p3.data.flip();
        assert_eq!(layer.paths[0], p3);
        p4.data.flip();
        assert_eq!(layer.paths[1], p4);
        assert_eq!(layer.paths[2], p2);
        assert_eq!(layer.paths[3], p1);
    }

    //#[ignore] // this test fails with kiddo 2.0.0-beta.5
    #[test]
    fn test_sort_problematic_case() {
        let mut doc = Document::from_svg(test_file!("random_100_sort.svg"), false).unwrap();
        doc.get_mut(1).sort(true, 1000);
    }
}
