use super::*;
use crate::{Layer, PageSize, Path};
use std::collections::BTreeMap;

#[derive(Default, Clone, Debug)]
pub struct Document {
    pub layers: BTreeMap<LayerID, Layer>,
    metadata: DocumentMetadata,
}

impl Transforms for Document {
    fn transform(&mut self, affine: &kurbo::Affine) {
        self.layers.iter_mut().for_each(|(_, layer)| {
            layer.transform(affine);
        });
    }
}

impl DocumentTrait<Layer, Path, kurbo::BezPath> for Document {
    fn layers(&self) -> &BTreeMap<LayerID, Layer> {
        &self.layers
    }

    fn layers_mut(&mut self) -> &mut BTreeMap<LayerID, Layer> {
        &mut self.layers
    }

    fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut DocumentMetadata {
        &mut self.metadata
    }
}

impl Document {
    #[must_use]
    pub fn new_with_page_size(page_size: PageSize) -> Self {
        let metadata = DocumentMetadata {
            page_size: Some(page_size),
            ..Default::default()
        };
        Self {
            metadata,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn flatten(&self, tolerance: f64) -> FlattenedDocument {
        FlattenedDocument::new(
            self.layers
                .iter()
                .map(|(id, layer)| (*id, layer.flatten(tolerance)))
                .collect(),
            self.metadata.with_source_suffix(" (flattened)"),
        )
    }

    #[must_use]
    pub fn control_points(&self) -> FlattenedDocument {
        FlattenedDocument::new(
            self.layers
                .iter()
                .map(|(id, layer)| (*id, layer.control_points()))
                .collect(),
            self.metadata.with_source_suffix(" (control points)"),
        )
    }

    pub fn crop(&mut self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> &Self {
        self.layers.iter_mut().for_each(|(_, layer)| {
            layer.crop(x_min, y_min, x_max, y_max);
        });
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Layer;

    #[test]
    fn test_document_bounds() {
        let mut doc = Document::new();
        assert_eq!(doc.bounds(), None);

        let layer1 = Layer::new();
        doc.layers.insert(1, layer1);
        assert_eq!(doc.bounds(), None);

        let mut layer2 = Layer::new();
        layer2
            .paths
            .push(kurbo::Line::new((10., 10.), (25., 53.)).into());
        let layer2_bounds = layer2.bounds();
        doc.layers.insert(2, layer2);
        assert_eq!(doc.bounds(), layer2_bounds);

        let mut layer3 = Layer::new();
        layer3
            .paths
            .push(kurbo::Line::new((25., -100.), (250., 54.)).into());
        doc.layers.insert(3, layer3);
        assert_eq!(doc.bounds(), Some(kurbo::Rect::new(10., -100., 250., 54.)));
    }
}
