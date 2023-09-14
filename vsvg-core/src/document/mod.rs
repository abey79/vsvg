use crate::{LayerTrait, PathDataTrait, PathTrait, Transforms};
use std::collections::BTreeMap;

#[allow(clippy::module_inception)]
mod document;
mod flattened_document;
mod metadata;

use crate::stats::LayerStats;
use crate::svg_writer::document_to_svg_string;
pub use document::Document;
pub use flattened_document::FlattenedDocument;
pub use metadata::DocumentMetadata;

pub type LayerID = usize;

pub trait DocumentTrait<L: LayerTrait<P, D>, P: PathTrait<D>, D: PathDataTrait>:
    Transforms
{
    fn layers(&self) -> &BTreeMap<LayerID, L>;

    fn layers_mut(&mut self) -> &mut BTreeMap<LayerID, L>;

    fn metadata(&self) -> &DocumentMetadata;

    fn metadata_mut(&mut self) -> &mut DocumentMetadata;

    fn try_get(&self, id: LayerID) -> Option<&L> {
        self.layers().get(&id)
    }

    #[must_use]
    fn get_mut(&mut self, id: LayerID) -> &mut L {
        self.layers_mut().entry(id).or_insert_with(|| L::default())
    }

    fn for_each<F>(&mut self, f: F)
    where
        F: Fn(&mut L),
    {
        self.layers_mut().values_mut().for_each(f);
    }

    #[must_use]
    fn bounds(&self) -> Option<kurbo::Rect> {
        if self.layers().is_empty() {
            return None;
        }

        let mut values = self.layers().values();
        let first = values.next().expect("not empty").bounds();
        values.fold(first, |acc, layer| match (acc, layer.bounds()) {
            (Some(acc), Some(layer)) => Some(acc.union(layer)),
            (Some(acc), None) => Some(acc),
            (None, Some(path)) => Some(path),
            (None, None) => None,
        })
    }

    #[must_use]
    fn stats(&self) -> BTreeMap<LayerID, LayerStats> {
        self.layers()
            .iter()
            .map(|(id, layer)| (*id, layer.stats()))
            .collect()
    }

    fn to_svg(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        document_to_svg_string(self, writer)
    }
}
