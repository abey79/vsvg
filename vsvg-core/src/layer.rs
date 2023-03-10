use crate::flattened_layer::FlattenedLayer;
use crate::path::{PathData, PathImpl};
use crate::{FlattenedPath, PathType};

pub type Layer = LayerImpl<PathData>;

#[derive(Default, Clone, Debug)]
pub struct LayerImpl<T: PathType> {
    pub paths: Vec<PathImpl<T>>,
    pub name: String,
}

impl<T: PathType> LayerImpl<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    #[must_use]
    pub fn bounds(&self) -> Option<kurbo::Rect> {
        if self.paths.is_empty() {
            return None;
        }

        let first = self.paths.first().expect("checked").bounds();
        Some(
            self.paths
                .iter()
                .skip(1)
                .fold(first, |acc, path| acc.union(path.bounds())),
        )
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

        FlattenedLayer {
            paths: flattened_paths,
            name: self.name.clone(),
        }
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

        layer.paths.push(PathImpl::from_shape(kurbo::Line::new(
            (0.0, 0.0),
            (10., 15.),
        )));
        assert_eq!(layer.bounds(), Some(kurbo::Rect::new(0.0, 0.0, 10.0, 15.0)));

        layer.paths.push(PathImpl::from_shape(kurbo::Line::new(
            (25.0, 53.0),
            (-10., -150.),
        )));
        assert_eq!(
            layer.bounds(),
            Some(kurbo::Rect::new(-10.0, -150.0, 25.0, 53.0))
        );
    }
}
