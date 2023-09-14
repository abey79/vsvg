use super::{FlattenedLayer, LayerMetadata, LayerTrait, Transforms};
use crate::{FlattenedPath, Path, Point};

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
    fn transform(&mut self, affine: &kurbo::Affine) {
        self.paths.iter_mut().for_each(|path| {
            path.transform(affine);
        });
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
        let flattened_paths =
            self.paths
                .iter()
                .fold(Vec::<FlattenedPath>::new(), |mut polylines, path| {
                    polylines.append(&mut path.flatten(tolerance));
                    polylines
                });

        FlattenedLayer::new(flattened_paths, self.metadata.clone())
    }

    #[must_use]
    pub fn control_points(&self) -> FlattenedLayer {
        FlattenedLayer::new(
            self.paths.iter().flat_map(Path::control_points).collect(),
            self.metadata.clone(),
        )
    }

    #[must_use]
    pub fn display_vertices(&self) -> Vec<Point> {
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
        self.paths.iter_mut().for_each(|path| {
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

        layer
            .paths
            .push(kurbo::Line::new((0.0, 0.0), (10., 15.)).into());
        assert_eq!(layer.bounds(), Some(kurbo::Rect::new(0.0, 0.0, 10.0, 15.0)));

        layer
            .paths
            .push(kurbo::Line::new((25.0, 53.0), (-10., -150.)).into());
        assert_eq!(
            layer.bounds(),
            Some(kurbo::Rect::new(-10.0, -150.0, 25.0, 53.0))
        );
    }
}
