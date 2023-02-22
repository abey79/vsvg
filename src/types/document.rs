use crate::types::{Layer, PageSize, Polylines};

#[derive(Default, Clone, Debug)]
pub struct Document {
    pub layers: Vec<Layer>,
    pub page_size: Option<PageSize>,
}

impl Document {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn new_with_page_size(page_size: PageSize) -> Self {
        Self {
            page_size: Some(page_size),
            ..Default::default()
        }
    }

    pub fn flatten(&self, tolerance: f64) -> Polylines {
        self.layers
            .iter()
            .fold(Polylines::new(), |mut polylines, layer| {
                polylines.append(&mut layer.flatten(tolerance));
                polylines
            })
    }

    pub fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        Self {
            layers: self
                .layers
                .into_iter()
                .map(|layer| layer.crop(x_min, y_min, x_max, y_max))
                .collect(),
            ..self
        }
    }
}
