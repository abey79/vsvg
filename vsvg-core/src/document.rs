use crate::flattened_layer::FlattenedLayer;
use crate::flattened_path::Polyline;
use crate::layer::LayerImpl;
use crate::path::PathData;
use crate::{PageSize, Path, PathType};
use std::collections::BTreeMap;

pub type LayerID = usize;

pub type Document = DocumentImpl<PathData>;
pub type FlattenedDocument = DocumentImpl<Polyline>;

#[derive(Default, Clone, Debug)]
pub struct DocumentImpl<T: PathType> {
    pub layers: BTreeMap<LayerID, LayerImpl<T>>,
    pub page_size: Option<PageSize>,
    pub source: Option<String>,
}

impl<T: PathType> DocumentImpl<T> {
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

    #[must_use]
    pub fn get_mut(&mut self, id: LayerID) -> &mut LayerImpl<T> {
        self.layers.entry(id).or_insert_with(|| LayerImpl::new())
    }

    pub fn for_each<F>(&mut self, f: F)
    where
        F: Fn(&mut LayerImpl<T>),
    {
        self.layers.values_mut().for_each(f);
    }

    #[must_use]
    pub fn bounds(&self) -> Option<kurbo::Rect> {
        if self.layers.is_empty() {
            return None;
        }

        let mut values = self.layers.values();
        let first = values.next().expect("not empty").bounds();
        values.fold(first, |acc, layer| match (acc, layer.bounds()) {
            (Some(acc), Some(layer)) => Some(acc.union(layer)),
            (Some(acc), None) => Some(acc),
            (None, Some(path)) => Some(path),
            (None, None) => None,
        })
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
            source: Some(format!(
                "{} (flattened)",
                self.source.as_deref().unwrap_or("<empty>")
            )),
        }
    }

    #[must_use]
    pub fn control_points(&self) -> FlattenedDocument {
        FlattenedDocument {
            layers: self
                .layers
                .iter()
                .map(|(id, layer)| {
                    (
                        *id,
                        FlattenedLayer {
                            paths: layer.paths.iter().flat_map(Path::control_points).collect(),
                            name: layer.name.clone(),
                        },
                    )
                })
                .collect(),
            page_size: self.page_size,
            source: Some(format!(
                "{} (control points)",
                self.source.as_deref().unwrap_or("<empty>")
            )),
        }
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
