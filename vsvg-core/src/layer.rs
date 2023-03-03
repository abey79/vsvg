use crate::flattened_layer::FlattenedLayer;
use crate::path::{PathData, PathImpl};
use crate::FlattenedPath;

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
