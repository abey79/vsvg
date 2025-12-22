//! Structure to hold the state for the [`Draw`] API between commands.

use kurbo::Affine;
use vsvg::{
    DEFAULT_TOLERANCE, Draw, IntoBezPathTolerance, Layer, Path, PathMetadata, PathTrait, Transforms,
};

#[derive(Debug)]
pub struct DrawState {
    pub transform: Affine,
    pub metadata: PathMetadata,

    /// used to convert shapes to Béziers
    pub tolerance: f64,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            transform: Affine::default(),
            metadata: PathMetadata::default(),
            tolerance: DEFAULT_TOLERANCE,
        }
    }
}

impl Transforms for DrawState {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        self.transform = *affine * self.transform;
        self
    }
}

pub struct LayerDrawer<'layer, 'state> {
    pub(crate) state: &'state DrawState,
    pub(crate) layer: &'layer mut Layer,
}

impl<'layer, 'state> Draw for LayerDrawer<'layer, 'state> {
    fn add_path<T: IntoBezPathTolerance>(&mut self, path: T) -> &mut Self {
        let mut path: Path = Path::from_tolerance(path, self.state.tolerance);
        *path.metadata_mut() = self.state.metadata.clone();
        path.apply_transform(self.state.transform);
        self.layer.paths.push(path);
        self
    }
}
