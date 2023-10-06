use crate::{LayerTrait, PathDataTrait, PathTrait, Transforms};
use std::collections::BTreeMap;

#[allow(clippy::module_inception)]
mod document;
mod flattened_document;
mod metadata;

use crate::document_to_svg_doc;
use crate::stats::LayerStats;
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

    fn push_path(&mut self, id: LayerID, path: impl Into<P>) {
        self.get_mut(id).push_path(path.into());
    }

    fn try_get(&self, id: LayerID) -> Option<&L> {
        self.layers().get(&id)
    }

    #[must_use]
    fn get_mut(&mut self, id: LayerID) -> &mut L {
        self.layers_mut().entry(id).or_insert_with(|| L::default())
    }

    fn ensure_exists(&mut self, id: LayerID) {
        let _ = self.get_mut(id);
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

    fn to_svg_string(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;

        let doc = document_to_svg_doc(self);
        let mut svg = String::new();
        write!(svg, "{doc}").map(|()| svg)
    }
    fn to_svg(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        let doc = document_to_svg_doc(self);
        svg::write(writer, &doc)
    }

    fn to_svg_file(&self, file_path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let file = std::io::BufWriter::new(std::fs::File::create(file_path)?);
        self.to_svg(file)
    }
}
