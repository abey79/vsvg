use super::*;
use crate::{FlattenedPath, Polyline};

#[derive(Default, Clone, Debug)]
pub struct FlattenedLayer {
    pub paths: Vec<FlattenedPath>,
    metadata: LayerMetadata,
}

impl FlattenedLayer {
    #[must_use]
    pub fn new(paths: Vec<FlattenedPath>, metadata: LayerMetadata) -> Self {
        Self { paths, metadata }
    }
}

impl Transforms for FlattenedLayer {
    fn transform(&mut self, affine: &kurbo::Affine) {
        self.paths.iter_mut().for_each(|path| {
            path.transform(affine);
        });
    }
}
impl LayerTrait<FlattenedPath, Polyline> for FlattenedLayer {
    fn paths(&self) -> &Vec<FlattenedPath> {
        &self.paths
    }

    fn metadata(&self) -> &LayerMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut LayerMetadata {
        &mut self.metadata
    }
}
