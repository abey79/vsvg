use super::{FlattenedLayer, LayerMetadata, LayerTrait, Transforms};
use crate::{Draw, IntoBezPathTolerance, Path, PathMetadata, PathTrait, Point};
use rayon::prelude::*;

#[derive(Default, Clone, Debug)]
pub struct Layer {
    pub paths: Vec<Path>,
    metadata: LayerMetadata,
}

/// Implementing this trait allows applying affine transforms to the layer content.
impl Transforms for Layer {
    fn transform(&mut self, affine: &kurbo::Affine) -> &mut Self {
        self.paths.par_iter_mut().for_each(|path| {
            path.transform(affine);
        });
        self
    }
}

impl LayerTrait<Path, kurbo::BezPath> for Layer {
    fn from_paths_and_metadata(paths: Vec<Path>, metadata: LayerMetadata) -> Self {
        Self { paths, metadata }
    }

    fn paths(&self) -> &[Path] {
        &self.paths
    }

    fn paths_mut(&mut self) -> &mut Vec<Path> {
        &mut self.paths
    }

    fn metadata(&self) -> &LayerMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut LayerMetadata {
        &mut self.metadata
    }
}

/// Implementing this trait allows drawing directly into a layer.
///
/// Each [`trait@Draw`] method will append a new path with default metadata to the layer.
///
/// # Example
///
/// ```
/// use vsvg::{Draw, DocumentTrait};
///
/// let mut doc = vsvg::Document::default();
/// let layer = doc.get_mut(0);
/// layer.circle(5.0, 5.0, 10.0);
/// ```
impl Draw for Layer {
    fn add_path<T: IntoBezPathTolerance>(&mut self, path: T) -> &mut Self {
        self.push_path(Path::from_metadata(path, PathMetadata::default()));
        self
    }
}

impl Layer {
    #[must_use]
    pub fn flatten(&self, tolerance: f64) -> FlattenedLayer {
        crate::trace_function!();

        let flattened_paths = self
            .paths
            .par_iter()
            .flat_map(|path| path.flatten(tolerance))
            .collect();

        FlattenedLayer::from_paths_and_metadata(flattened_paths, self.metadata.clone())
    }

    #[must_use]
    pub fn bezier_handles(&self) -> FlattenedLayer {
        crate::trace_function!();

        FlattenedLayer::from_paths_and_metadata(
            self.paths
                .par_iter()
                .flat_map(Path::bezier_handles)
                .collect(),
            self.metadata.clone(),
        )
    }

    #[must_use]
    pub fn display_vertices(&self) -> Vec<Point> {
        crate::trace_function!();

        self.paths
            .iter()
            .flat_map(|path| {
                path.data.iter().filter_map(|el| match el {
                    kurbo::PathEl::MoveTo(pt)
                    | kurbo::PathEl::LineTo(pt)
                    | kurbo::PathEl::CurveTo(_, _, pt)
                    | kurbo::PathEl::QuadTo(_, pt) => Some(pt.into()),
                    kurbo::PathEl::ClosePath => None,
                })
            })
            .collect()
    }

    pub fn crop(
        &mut self,
        x_min: impl Into<f64>,
        y_min: impl Into<f64>,
        x_max: impl Into<f64>,
        y_max: impl Into<f64>,
    ) -> &Self {
        let x_min = x_min.into();
        let y_min = y_min.into();
        let x_max = x_max.into();
        let y_max = y_max.into();

        self.paths.par_iter_mut().for_each(|path| {
            path.crop(x_min, y_min, x_max, y_max);
        });

        self
    }

    /// Promote common metadata from paths to layer defaults.
    ///
    /// If all paths share the same value for a metadata field, that value is
    /// moved to `default_path_metadata` and cleared (`None`) from individual paths.
    /// This reduces redundancy and produces smaller SVG files on export.
    ///
    /// This is typically called after loading an SVG, since parsers like usvg
    /// resolve inherited styles onto each path individually.
    pub fn promote_common_metadata_to_defaults(&mut self) {
        if self.paths.is_empty() {
            return;
        }

        let PathMetadata {
            color: first_color,
            stroke_width: first_width,
        } = self.paths[0].metadata().clone();

        // Check if all paths have the same color
        if first_color.is_some() {
            let all_same_color = self.paths.iter().all(|p| p.metadata().color == first_color);
            if all_same_color {
                self.metadata.default_path_metadata.color = first_color;
                for path in &mut self.paths {
                    path.metadata_mut().color = None;
                }
            }
        }

        // Check if all paths have the same stroke_width
        if first_width.is_some() {
            let all_same_width =
                self.paths
                    .iter()
                    .all(|p| match (p.metadata().stroke_width, first_width) {
                        (Some(a), Some(b)) => (a - b).abs() < f64::EPSILON,
                        _ => false,
                    });
            if all_same_width {
                self.metadata.default_path_metadata.stroke_width = first_width;
                for path in &mut self.paths {
                    path.metadata_mut().stroke_width = None;
                }
            }
        }
    }
}

impl From<FlattenedLayer> for Layer {
    fn from(flattened_layer: FlattenedLayer) -> Self {
        Self {
            paths: flattened_layer.paths.into_iter().map(Path::from).collect(),
            metadata: flattened_layer.metadata,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Color, PathTrait};

    #[test]
    fn test_layer_bounds() {
        let mut layer = Layer::new();
        assert_eq!(layer.bounds(), None);

        layer.push_path(Path::from(kurbo::Line::new((0.0, 0.0), (10., 15.))));
        assert_eq!(layer.bounds(), Some(kurbo::Rect::new(0.0, 0.0, 10.0, 15.0)));

        layer.push_path(Path::from(kurbo::Line::new((25.0, 53.0), (-10., -150.))));
        assert_eq!(
            layer.bounds(),
            Some(kurbo::Rect::new(-10.0, -150.0, 25.0, 53.0))
        );
    }

    #[test]
    fn test_layer_push_shape() {
        let mut layer = Layer::new();
        layer.push_path(kurbo::Rect::new(0.0, 0.0, 10.0, 10.0));
        assert_eq!(layer.paths.len(), 1);
        assert_eq!(
            layer.paths[0],
            Path::from(kurbo::Rect::new(0.0, 0.0, 10.0, 10.0))
        );
    }

    #[test]
    fn test_promote_common_color() {
        let mut layer = Layer::new();

        // Add paths with the same color
        let mut path1 = Path::from(kurbo::Line::new((0.0, 0.0), (10., 10.)));
        path1.metadata_mut().color = Some(Color::RED);
        let mut path2 = Path::from(kurbo::Line::new((10.0, 10.0), (20., 20.)));
        path2.metadata_mut().color = Some(Color::RED);
        layer.push_path(path1);
        layer.push_path(path2);

        layer.promote_common_metadata_to_defaults();

        // Color should be promoted to layer defaults
        assert_eq!(
            layer.metadata().default_path_metadata.color,
            Some(Color::RED)
        );
        // And cleared from paths
        assert_eq!(layer.paths[0].metadata().color, None);
        assert_eq!(layer.paths[1].metadata().color, None);
    }

    #[test]
    fn test_promote_common_stroke_width() {
        let mut layer = Layer::new();

        // Add paths with the same stroke_width
        let mut path1 = Path::from(kurbo::Line::new((0.0, 0.0), (10., 10.)));
        path1.metadata_mut().stroke_width = Some(2.5);
        let mut path2 = Path::from(kurbo::Line::new((10.0, 10.0), (20., 20.)));
        path2.metadata_mut().stroke_width = Some(2.5);
        layer.push_path(path1);
        layer.push_path(path2);

        layer.promote_common_metadata_to_defaults();

        // stroke_width should be promoted to layer defaults
        assert_eq!(
            layer.metadata().default_path_metadata.stroke_width,
            Some(2.5)
        );
        // And cleared from paths
        assert_eq!(layer.paths[0].metadata().stroke_width, None);
        assert_eq!(layer.paths[1].metadata().stroke_width, None);
    }

    #[test]
    fn test_no_promote_when_different_colors() {
        let mut layer = Layer::new();

        // Add paths with different colors
        let mut path1 = Path::from(kurbo::Line::new((0.0, 0.0), (10., 10.)));
        path1.metadata_mut().color = Some(Color::RED);
        let mut path2 = Path::from(kurbo::Line::new((10.0, 10.0), (20., 20.)));
        path2.metadata_mut().color = Some(Color::BLUE);
        layer.push_path(path1);
        layer.push_path(path2);

        layer.promote_common_metadata_to_defaults();

        // Color should NOT be promoted (different values)
        assert_eq!(layer.metadata().default_path_metadata.color, None);
        // Paths keep their colors
        assert_eq!(layer.paths[0].metadata().color, Some(Color::RED));
        assert_eq!(layer.paths[1].metadata().color, Some(Color::BLUE));
    }

    #[test]
    fn test_promote_empty_layer() {
        let mut layer = Layer::new();
        // Should not panic on empty layer
        layer.promote_common_metadata_to_defaults();
        assert_eq!(layer.metadata().default_path_metadata.color, None);
    }

    #[test]
    fn test_promote_partial_metadata() {
        let mut layer = Layer::new();

        // All paths have same color, but different stroke_widths
        let mut path1 = Path::from(kurbo::Line::new((0.0, 0.0), (10., 10.)));
        path1.metadata_mut().color = Some(Color::GREEN);
        path1.metadata_mut().stroke_width = Some(1.0);
        let mut path2 = Path::from(kurbo::Line::new((10.0, 10.0), (20., 20.)));
        path2.metadata_mut().color = Some(Color::GREEN);
        path2.metadata_mut().stroke_width = Some(2.0);
        layer.push_path(path1);
        layer.push_path(path2);

        layer.promote_common_metadata_to_defaults();

        // Color should be promoted (same)
        assert_eq!(
            layer.metadata().default_path_metadata.color,
            Some(Color::GREEN)
        );
        assert_eq!(layer.paths[0].metadata().color, None);
        assert_eq!(layer.paths[1].metadata().color, None);

        // stroke_width should NOT be promoted (different)
        assert_eq!(layer.metadata().default_path_metadata.stroke_width, None);
        assert_eq!(layer.paths[0].metadata().stroke_width, Some(1.0));
        assert_eq!(layer.paths[1].metadata().stroke_width, Some(2.0));
    }
}
