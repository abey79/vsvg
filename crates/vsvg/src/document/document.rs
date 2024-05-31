use super::{DocumentMetadata, DocumentTrait, FlattenedDocument, LayerID};
use crate::{Layer, PageSize, Path, Transforms};
use std::collections::BTreeMap;

#[derive(Default, Clone, Debug)]
pub struct Document {
    pub layers: BTreeMap<LayerID, Layer>,
    metadata: DocumentMetadata,
}

impl Transforms for Document {
    fn transform(&mut self, affine: &kurbo::Affine) -> &mut Self {
        self.layers.iter_mut().for_each(|(_, layer)| {
            layer.transform(affine);
        });
        self
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
        crate::trace_function!();

        FlattenedDocument::new(
            self.layers
                .iter()
                .map(|(id, layer)| (*id, layer.flatten(tolerance)))
                .collect(),
            self.metadata.with_source_suffix(" (flattened)"),
        )
    }

    #[must_use]
    pub fn bezier_handles(&self) -> FlattenedDocument {
        crate::trace_function!();

        FlattenedDocument::new(
            self.layers
                .iter()
                .map(|(id, layer)| (*id, layer.bezier_handles()))
                .collect(),
            self.metadata.with_source_suffix(" (control points)"),
        )
    }

    /// Crops the contents to the bounds provided.
    pub fn crop(
        &mut self,
        x_min: impl Into<f64>,
        y_min: impl Into<f64>,
        x_max: impl Into<f64>,
        y_max: impl Into<f64>,
    ) {
        let x_min = x_min.into();
        let y_min = y_min.into();
        let x_max = x_max.into();
        let y_max = y_max.into();

        self.layers.iter_mut().for_each(|(_, layer)| {
            layer.crop(x_min, y_min, x_max, y_max);
        });
    }

    /// Translates the content of the document so that it's centered on the page.
    ///
    /// If the document has no page size defined, the content is translated such that its bounds
    /// become `(0, 0, content_width, content_height)`.
    pub fn center_content(&mut self) {
        let Some(bounds) = self.bounds() else {
            return;
        };

        let (dx, dy) = if let Some(page_size) = self.metadata().page_size {
            let (w, h) = page_size.to_pixels();
            let content_width = bounds.width();
            let content_height = bounds.height();

            (
                (w - content_width) / 2. - bounds.x0,
                (h - content_height) / 2. - bounds.y0,
            )
        } else {
            (-bounds.x0, -bounds.y0)
        };

        self.translate(dx, dy);
    }
}

impl From<FlattenedDocument> for Document {
    fn from(flattened_document: FlattenedDocument) -> Self {
        Self {
            layers: flattened_document
                .layers
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),

            metadata: flattened_document.metadata,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Layer, LayerTrait, Unit};

    #[test]
    fn test_document_bounds() {
        let mut doc = Document::default();
        assert_eq!(doc.bounds(), None);

        let layer1 = Layer::new();
        doc.layers.insert(1, layer1);
        assert_eq!(doc.bounds(), None);

        let mut layer2 = Layer::new();
        layer2
            .paths
            .push(Path::from(kurbo::Line::new((10., 10.), (25., 53.))));
        let layer2_bounds = layer2.bounds();
        doc.layers.insert(2, layer2);
        assert_eq!(doc.bounds(), layer2_bounds);

        let mut layer3 = Layer::new();
        layer3
            .paths
            .push(Path::from(kurbo::Line::new((25., -100.), (250., 54.))));
        doc.layers.insert(3, layer3);
        assert_eq!(doc.bounds(), Some(kurbo::Rect::new(10., -100., 250., 54.)));
    }

    #[test]
    fn test_document_push_shape() {
        let mut doc = Document::default();
        doc.push_path(2, kurbo::Rect::new(0., 0., 10., 10.));

        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.layers[&2].paths.len(), 1);
        assert_eq!(
            doc.layers[&2].paths[0],
            Path::from(kurbo::Rect::new(0., 0., 10., 10.))
        );
    }

    #[test]
    fn test_document_center_no_page_size() {
        let mut doc = Document::default();
        doc.push_path(2, kurbo::Line::new((10., 10.), (25., 53.)));
        doc.center_content();
        assert_eq!(doc.bounds(), Some(kurbo::Rect::new(0., 0., 15., 43.)));
    }

    #[test]
    fn test_document_center_with_page_size() {
        let mut doc = Document::default();
        doc.metadata_mut().page_size = Some(PageSize::Custom(300., 200., Unit::Px));
        doc.push_path(2, kurbo::Line::new((10., 10.), (30., 70.)));
        doc.center_content();
        assert_eq!(doc.bounds(), Some(kurbo::Rect::new(140., 70.0, 160., 130.)));
    }
}
