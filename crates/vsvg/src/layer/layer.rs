use super::{FlattenedLayer, LayerMetadata, LayerTrait, Transforms};
use crate::{Path, Point};
use rayon::prelude::*;

#[derive(Default, Clone, Debug)]
pub struct Layer {
    pub paths: Vec<Path>,
    metadata: LayerMetadata,
}

impl Layer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Transforms for Layer {
    fn transform(&mut self, affine: &kurbo::Affine) -> &mut Self {
        self.paths.par_iter_mut().for_each(|path| {
            path.transform(affine);
        });
        self
    }
}

impl LayerTrait<Path, kurbo::BezPath> for Layer {
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

impl Layer {
    #[must_use]
    pub fn flatten(&self, tolerance: f64) -> FlattenedLayer {
        crate::trace_function!();

        let flattened_paths = self
            .paths
            .par_iter()
            .flat_map(|path| path.flatten(tolerance))
            .collect();

        FlattenedLayer::new(flattened_paths, self.metadata.clone())
    }

    #[must_use]
    pub fn bezier_handles(&self) -> FlattenedLayer {
        crate::trace_function!();

        FlattenedLayer::new(
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

    pub fn crop(&mut self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> &Self {
        self.paths.par_iter_mut().for_each(|path| {
            path.crop(x_min, y_min, x_max, y_max);
        });

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
}
