use crate::types::flattened_layer::FlattenedLayer;
use crate::types::path::{PathData, PathImpl};
use crate::types::FlattenedPath;

pub type Layer = LayerImpl<PathData>;

#[derive(Default, Clone, Debug)]
pub struct LayerImpl<T: Default> {
    pub paths: Vec<PathImpl<T>>,
    pub name: String,
}

impl<T: Default> LayerImpl<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub(crate) fn map_paths(self, f: impl Fn(PathImpl<T>) -> PathImpl<T>) -> Self {
        Self {
            paths: self.paths.into_iter().map(f).collect(),
            ..self
        }
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

    #[must_use]
    pub fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        self.map_paths(|path| path.crop(x_min, y_min, x_max, y_max))
    }
}
