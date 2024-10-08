use super::{LayerMetadata, LayerTrait};
use crate::{FlattenedPath, Polyline, Transforms};

#[derive(Default, Clone, Debug)]
pub struct FlattenedLayer {
    pub paths: Vec<FlattenedPath>,
    pub(crate) metadata: LayerMetadata,
}

impl FlattenedLayer {
    #[must_use]
    pub fn vertex_count(&self) -> u64 {
        self.paths
            .iter()
            .map(|path| path.data.points().len() as u64)
            .sum()
    }
}

impl Transforms for FlattenedLayer {
    fn transform(&mut self, affine: &kurbo::Affine) -> &mut Self {
        self.paths.iter_mut().for_each(|path| {
            path.transform(affine);
        });
        self
    }
}
impl LayerTrait<FlattenedPath, Polyline> for FlattenedLayer {
    fn from_paths_and_metadata(paths: Vec<FlattenedPath>, metadata: LayerMetadata) -> Self {
        Self { paths, metadata }
    }

    fn paths(&self) -> &[FlattenedPath] {
        &self.paths
    }

    fn paths_mut(&mut self) -> &mut Vec<FlattenedPath> {
        &mut self.paths
    }

    fn metadata(&self) -> &LayerMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut LayerMetadata {
        &mut self.metadata
    }
}
