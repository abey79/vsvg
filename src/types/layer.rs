use crate::types::{Path, Polylines};

#[derive(Default, Clone, Debug)]
pub struct Layer {
    pub paths: Vec<Path>,
}

impl Layer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn flatten(&self, tolerance: f64) -> Polylines {
        self.paths
            .iter()
            .fold(Polylines::new(), |mut polylines, path| {
                polylines.append(&mut path.flatten(tolerance));
                polylines
            })
    }

    pub fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        Self {
            paths: self
                .paths
                .into_iter()
                .map(|path| path.crop(x_min, y_min, x_max, y_max))
                .collect(),
        }
    }
}
