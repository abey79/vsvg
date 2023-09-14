use super::{DocumentMetadata, DocumentTrait, LayerID};
use crate::{FlattenedLayer, FlattenedPath, Polyline, Transforms};
use std::collections::BTreeMap;

#[derive(Default, Clone, Debug)]
pub struct FlattenedDocument {
    pub layers: BTreeMap<LayerID, FlattenedLayer>,
    metadata: DocumentMetadata,
}

impl FlattenedDocument {
    #[must_use]
    pub fn new(layers: BTreeMap<LayerID, FlattenedLayer>, metadata: DocumentMetadata) -> Self {
        Self { layers, metadata }
    }
}

impl Transforms for FlattenedDocument {
    fn transform(&mut self, affine: &kurbo::Affine) {
        self.layers.iter_mut().for_each(|(_, layer)| {
            layer.transform(affine);
        });
    }
}

impl DocumentTrait<FlattenedLayer, FlattenedPath, Polyline> for FlattenedDocument {
    fn layers(&self) -> &BTreeMap<LayerID, FlattenedLayer> {
        &self.layers
    }

    fn layers_mut(&mut self) -> &mut BTreeMap<LayerID, FlattenedLayer> {
        &mut self.layers
    }

    fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut DocumentMetadata {
        &mut self.metadata
    }
}
