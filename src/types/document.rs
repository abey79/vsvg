use crate::types::flattened_path::Polyline;
use crate::types::layer::LayerImpl;
use crate::types::path::PathData;
use crate::types::PageSize;
use std::collections::BTreeMap;

pub type LayerID = usize;

pub type Document = DocumentImpl<PathData>;
pub type FlattenedDocument = DocumentImpl<Polyline>;

#[derive(Default, Clone, Debug)]
pub struct DocumentImpl<T: Default> {
    pub layers: BTreeMap<LayerID, LayerImpl<T>>,
    pub page_size: Option<PageSize>,
}

impl<T: Default> DocumentImpl<T> {
    #[allow(dead_code)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    #[must_use]
    pub fn try_get(&self, id: LayerID) -> Option<&LayerImpl<T>> {
        self.layers.get(&id)
    }

    pub fn get_mut(&mut self, id: LayerID) -> &mut LayerImpl<T> {
        self.layers.entry(id).or_insert_with(|| LayerImpl::new())
    }

    pub(crate) fn map_layers(self, f: impl Fn(LayerImpl<T>) -> LayerImpl<T>) -> Self {
        Self {
            layers: self.layers.into_iter().map(|(k, v)| (k, f(v))).collect(),
            ..self
        }
    }
}

impl Document {
    #[must_use]
    pub fn new_with_page_size(page_size: PageSize) -> Self {
        Self {
            page_size: Some(page_size),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn flatten(&self, tolerance: f64) -> FlattenedDocument {
        FlattenedDocument {
            layers: self
                .layers
                .iter()
                .map(|(id, layer)| (*id, layer.flatten(tolerance)))
                .collect(),
            page_size: self.page_size,
        }
    }

    #[must_use]
    pub fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        self.map_layers(|layer| layer.crop(x_min, y_min, x_max, y_max))
    }
}
