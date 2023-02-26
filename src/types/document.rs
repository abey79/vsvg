use crate::types::flattened_path::Polyline;
use crate::types::layer::LayerImpl;
use crate::types::path::PathData;
use crate::types::PageSize;

pub type Document = DocumentImpl<PathData>;
pub type FlattenedDocument = DocumentImpl<Polyline>;

#[derive(Default, Clone, Debug)]
pub struct DocumentImpl<T: Default> {
    pub layers: Vec<LayerImpl<T>>,
    pub page_size: Option<PageSize>,
}

impl<T: Default> DocumentImpl<T> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub(crate) fn map_layers(self, f: impl Fn(LayerImpl<T>) -> LayerImpl<T>) -> Self {
        Self {
            layers: self.layers.into_iter().map(f).collect(),
            ..self
        }
    }
}

impl Document {
    pub fn new_with_page_size(page_size: PageSize) -> Self {
        Self {
            page_size: Some(page_size),
            ..Default::default()
        }
    }

    pub fn flatten(&self, tolerance: f64) -> FlattenedDocument {
        FlattenedDocument {
            layers: self
                .layers
                .iter()
                .map(|layer| layer.flatten(tolerance))
                .collect(),
            page_size: self.page_size,
        }
        // self.layers
        //     .iter()
        //     .fold(Polylines::new(), |mut polylines, layer| {
        //         polylines.append(&mut layer.flatten(tolerance));
        //         polylines
        //     })
    }

    pub fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        self.map_layers(|layer| layer.crop(x_min, y_min, x_max, y_max))
    }
}
