//!
use kurbo::Affine;
use vsvg::path::IntoBezPathTolerance;
use vsvg::{
    Document, DocumentTrait, LayerID, PageSize, Path, PathMetadata, Transforms, DEFAULT_TOLERANCE,
};

/// Primary interface for drawing.
///
/// The [`Sketch`] type is the primary interface for drawing. To that effect, it implements the key
/// traits [`vsvg::Draw`] and [`vsvg::Transforms`].
///
/// # Drawing
///
/// The [`vsvg::Draw`] trait provide a wealth of chainable drawing functions, such as
/// [`vsvg::Draw::circle`]:
///
/// ```rust
/// # use whiskers::prelude::*;
/// # let mut sketch = Sketch::new();
/// sketch.circle(0.0, 0.0, 10.0).rect(10.0, 20.0, 20.0, 20.0);
/// ```
///
/// In addition to the basic, primitive draw calls, the [`vsvg::Draw`] trait also provides a more
/// flexible [`vsvg::Draw::add_path`] function that accepts any type which implements the
/// [`IntoBezPathTolerance`] trait. This currently includes many types from the [`::kurbo`] and
/// [`vsvg::geo`] crates.
///
/// # Transformations
///
/// The [`Sketch`] type implements the [`Transforms`] trait, which provides a number of chainable
/// affine transform functions such as [`Sketch::translate`] and [`Sketch::scale`]. In the context
/// of a sketch, these functions modify the current transform matrix, which affects _subsequent_
/// draw calls:
///
/// ```rust
/// # use whiskers::prelude::*;
/// # let mut sketch = Sketch::new();
/// sketch
///     .circle(0.0, 0.0, 10.0)  // not translated
///     .translate(5.0, 5.0)
///     .circle(0.0, 0.0, 10.0); // translated
/// ```
///
/// The [`Sketch`] type also maintains a stack of transform matrices, to make it easy to save and
/// restore current transform matrix with the [`Sketch::push_matrix`] and [`Sketch::pop_matrix`]:
///
/// ```rust
/// # use whiskers::prelude::*;
/// # let mut sketch = Sketch::new();
/// sketch
///     .circle(0.0, 0.0, 10.0)  // not translated
///     .push_matrix()
///     .translate(5.0, 5.0)
///     .circle(0.0, 0.0, 10.0)  // translated
///     .pop_matrix()
///     .circle(0.0, 0.0, 5.0);  // not translated
/// ```
///
/// The [`Sketch::push_matrix_and`] function is a convenience method to automatically save and
/// restore the current transform matrix around some draw calls:
///
/// ```rust
/// # use whiskers::prelude::*;
/// # let mut sketch = Sketch::new();
/// sketch
///     .circle(0.0, 0.0, 10.0)        // not translated
///     .push_matrix_and(|sketch| {
///         sketch.translate(5.0, 5.0)
///         .circle(0.0, 0.0, 10.0);   // translated
///     })
///     .circle(0.0, 0.0, 5.0);        // not translated
/// ```
///
/// # Runner use
///
/// In interactive sketches, the [`Sketch`] instance is constructed by the [`crate::Runner`] and
/// passed to the [`crate::App::update`] function. The runner takes care of configuring the page
/// size according to the UI.
///
/// # Standalone use
///
/// Alternatively, the [`Sketch`] type can be used as standalone object to build, display, and/or
/// export drawings to SVG:
///
/// ```no_run
/// use whiskers::prelude::*;
///
/// fn main() -> Result {
///    let mut sketch = Sketch::new();
///
///     sketch
///         .scale(2.0 * Unit::Cm)
///         .translate(10.0, 10.0)
///         .circle(0.0, 0.0, 3.0)
///         .rotate_deg(45.0)
///         .rect(0.0, 0.0, 6.5, 0.5)
///         .show()?
///         .save("circle.svg")?;
///
///     Ok(())
/// }
/// ```
///
/// This results in the following SVG file:
///
/// ![result](https://github.com/abey79/vsvg/assets/49431240/09d88775-0ad3-4776-be4b-b3872ba467b0)
///
/// Note that here the page size is not set. If needed, it must be set manually using the
/// [`Sketch::page_size`] function.

pub struct Sketch {
    document: Document,
    transform_stack: Vec<Affine>,
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
    /// Create a new, empty [`Sketch`].
    pub fn new() -> Self {
        Self::with_document(Document::default())
    }

    /// Create a [`Sketch`] from an existing [`Document`].
    pub fn with_document(mut document: Document) -> Self {
        let target_layer = 0;
        document.ensure_exists(target_layer);

        Self {
            document,
            tolerance: DEFAULT_TOLERANCE,
            transform_stack: vec![Affine::default()],
            target_layer,
            path_metadata: PathMetadata::default(),
        }
    }

    /// Sets the target layer for subsequent draw calls.
    pub fn set_layer(&mut self, layer_id: LayerID) -> &mut Self {
        self.document.ensure_exists(layer_id);
        self.target_layer = layer_id;
        self
    }

    /// Returns the sketch's width in pixels.
    ///
    /// If the page size is not set, it defaults to 400px.
    pub fn width(&self) -> f64 {
        self.document.metadata().page_size.map_or(400.0, |p| p.w())
    }

    /// Returns the sketch's height in pixels.
    ///
    /// If the page size is not set, it defaults to 400px.
    pub fn height(&self) -> f64 {
        self.document.metadata().page_size.map_or(400.0, |p| p.h())
    }

    /// Sets the [`Sketch`]'s page size.
    pub fn page_size(&mut self, page_size: PageSize) -> &mut Self {
        self.document.metadata_mut().page_size = Some(page_size);
        self
    }

    /// Sets the path color for subsequent draw calls.
    pub fn color(&mut self, color: impl Into<vsvg::Color>) -> &mut Self {
        self.path_metadata.color = color.into();
        self
    }

    /// Sets the path stroke width for subsequent draw calls.
    pub fn stroke_width(&mut self, width: impl Into<f64>) -> &mut Self {
        self.path_metadata.stroke_width = width.into();
        self
    }

    /// Push the current matrix onto the stack.
    ///
    /// A copy of the current transform matrix is pushed onto the stack. Use this before applying
    /// temporary transforms that you want to revert later with [`Sketch::pop_matrix`].
    pub fn push_matrix(&mut self) -> &mut Self {
        self.transform_stack
            .push(self.transform_stack.last().copied().unwrap_or_default());
        self
    }

    /// Push the identity matrix onto the stack.
    ///
    /// Use this if you want to temporarily reset the transform matrix and later revert to the
    /// current matrix with [`Sketch::pop_matrix`].
    pub fn push_matrix_reset(&mut self) -> &mut Self {
        self.transform_stack.push(Affine::default());
        self
    }

    /// Pop the current transform matrix from the stack, restoring the previously pushed matrix.
    pub fn pop_matrix(&mut self) -> &mut Self {
        if self.transform_stack.len() == 1 {
            log::warn!("pop_matrix: stack underflow");
            return self;
        }

        self.transform_stack.pop();
        self
    }

    /// Push the current matrix onto the stack, apply the given function, then pop the matrix.
    ///
    /// This is a convenience method for draw code that require a temporary change of the current
    /// transform matrix.
    pub fn push_matrix_and(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.push_matrix();
        f(self);
        self.pop_matrix();
        self
    }

    /// Centers the content of the sketch on the page, if the page size is set.
    ///
    /// **Note**: contrary to most other functions, this function is applied on the _existing_
    /// sketch content, not on subsequent draw calls.
    pub fn center(&mut self) -> &mut Self {
        self.document_mut().center_content();
        self
    }

    /// Returns a reference to the underlying [`Document`].
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Returns a mutable reference to the underlying [`Document`].
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }

    /// Opens the `vsvg` viewer with the sketch content.
    ///
    /// Requires the `viewer` feature to be enabled.
    #[cfg(feature = "viewer")]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn show(&mut self) -> anyhow::Result<&mut Self> {
        vsvg_viewer::show(self.document())?;
        Ok(self)
    }

    /// Saves the sketch content to an SVG file.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let file = std::io::BufWriter::new(std::fs::File::create(path)?);
        self.document.to_svg(file)?;
        Ok(())
    }
}

impl Transforms for Sketch {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        if let Some(matrix) = self.transform_stack.last_mut() {
            *matrix *= *affine;
        } else {
            log::warn!("transform: no matrix on the stack");
        }

        self
    }
}

impl vsvg::Draw for Sketch {
    fn add_path<T: IntoBezPathTolerance>(&mut self, path: T) -> &mut Self {
        let mut path: Path =
            Path::from_tolerance_metadata(path, self.tolerance, self.path_metadata.clone());

        if let Some(&matrix) = self.transform_stack.last() {
            path.apply_transform(matrix);
        } else {
            log::warn!("add_path: no matrix on the stack");
        }

        self.document.push_path(self.target_layer, path);
        self
    }
}
