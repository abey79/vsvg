use kurbo::Affine;
use vsvg::path::IntoBezPathTolerance;
use vsvg::{
    Document, DocumentTrait, Draw, LayerID, PageSize, Path, PathMetadata, Transforms,
    DEFAULT_TOLERANCE,
};

pub struct Sketch {
    document: Document,
    transform: Affine,
    target_layer: LayerID,
    tolerance: f64,
    path_metadata: PathMetadata,
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

        Self {
            document,
            tolerance: DEFAULT_TOLERANCE,
            transform: Affine::default(),
            target_layer,
            path_metadata: PathMetadata::default(),
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

    pub fn show(&mut self) -> anyhow::Result<&mut Self> {
        vsvg_viewer::show(self.document())?;
        Ok(self)
    }

    pub fn save(&mut self, path: impl AsRef<std::path::Path>) -> anyhow::Result<&mut Self> {
        let file = std::io::BufWriter::new(std::fs::File::create(path)?);
        self.document.to_svg(file)?;
        Ok(self)
    }
}

impl Transforms for Sketch {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        self.transform *= *affine;
        self
    }
}

impl Draw for Sketch {
    fn add_path<T: IntoBezPathTolerance>(&mut self, path: T) -> &mut Self {
        let mut path: Path =
            Path::from_tolerance_metadata(path, self.tolerance, self.path_metadata.clone());

        path.apply_transform(self.transform);

        self.document.push_path(self.target_layer, path);
        self
    }
}
