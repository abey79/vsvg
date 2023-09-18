use kurbo::Affine;
use vsvg::path::IntoBezPathTolerance;
use vsvg::{
    Document, DocumentTrait, DrawAPI, DrawState, LayerID, PageSize, Path, PathTrait, Transforms,
};

pub struct Sketch {
    document: Document,
    pub state: DrawState, //TODO: get rid of that!!
    transform: Affine,
    target_layer: LayerID,
}

impl Default for Sketch {
    fn default() -> Self {
        Self::new()
    }
}
impl Sketch {
    pub fn new() -> Self {
        Self::with_document(Document::default())
    }

    pub fn with_page_size(page_size: PageSize) -> Self {
        Self::with_document(Document::new_with_page_size(page_size))
    }

    pub fn with_document(mut document: Document) -> Self {
        let target_layer = 0;
        document.ensure_exists(target_layer);
        let state = DrawState::default();

        Self {
            document,
            state,
            transform: Affine::default(),
            target_layer,
        }
    }

    pub fn set_layer(&mut self, layer_id: LayerID) -> &mut Self {
        self.document.ensure_exists(layer_id);
        self.target_layer = layer_id;
        self
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn show(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        vsvg_viewer::show(self.document())?;
        Ok(self)
    }
}

impl Transforms for Sketch {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        self.transform *= *affine;
        self
    }
}

impl DrawAPI for Sketch {
    fn add_path<T: IntoBezPathTolerance>(&mut self, path: T) -> &mut Self {
        let mut path: Path =
            Path::from_tolerance_metadata(path, self.state.tolerance, self.state.metadata.clone());

        path.apply_transform(self.transform);

        self.document.push_path(self.target_layer, path);
        self
    }
}
