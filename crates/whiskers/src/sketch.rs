use std::collections::HashMap;

use kurbo::Affine;
use vsvg::{
    DEFAULT_TOLERANCE, Document, DocumentTrait, IntoBezPathTolerance, LayerID, LayerTrait, Length,
    PageSize, Path, PathDataTrait, PathMetadata, PathTrait, Transforms,
};

/// Captured style state for push/pop operations.
#[derive(Clone)]
struct StyleState {
    stroke_layer: Option<LayerID>,
    fill_layer: Option<LayerID>,
    path_metadata: PathMetadata,
}

/// Primary interface for drawing.
///
/// The [`Sketch`] type is the primary interface for drawing. To that effect, it implements the key
/// traits [`vsvg::Draw`] and [`Transforms`].
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
/// [`vsvg::exports::geo`] crates.
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
///         .rotate(Angle::from_deg(45.0))
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
    style_stack: Vec<StyleState>,
    tolerance: f64,
    path_metadata: PathMetadata,

    // Layer routing
    stroke_layer: Option<LayerID>,
    fill_layer: Option<LayerID>,

    // Per-layer hatch angles (in radians)
    hatch_angles: HashMap<LayerID, f64>,
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
        document.ensure_exists(0);

        Self {
            document,
            tolerance: DEFAULT_TOLERANCE,
            transform_stack: vec![Affine::default()],
            style_stack: vec![],
            // Default: black, 1px stroke width
            path_metadata: PathMetadata::default()
                .with_color(vsvg::Color::BLACK)
                .with_stroke_width(1.0),
            // Default: stroke to layer 0, no fill
            stroke_layer: Some(0),
            fill_layer: None,
            hatch_angles: HashMap::new(),
        }
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
        self.path_metadata.color = Some(color.into());
        self
    }

    /// Sets the path stroke width for subsequent draw calls.
    pub fn stroke_width(&mut self, width: impl Into<f64>) -> &mut Self {
        self.path_metadata.stroke_width = Some(width.into());
        self
    }

    // =========================================================================
    // Layer configuration
    // =========================================================================

    /// Configure a layer's properties.
    ///
    /// Returns a [`LayerHandle`] for setting `pen_width`, `color`, `name`, and `hatch_angle`.
    ///
    /// # Example
    /// ```
    /// # use whiskers::prelude::*;
    /// # let mut sketch = Sketch::new();
    /// sketch
    ///     .layer(0).pen_width(0.5 * Unit::Mm).color(Color::BLACK)
    ///     .layer(1).pen_width(0.3 * Unit::Mm).hatch_angle(std::f64::consts::FRAC_PI_4);
    /// ```
    pub fn layer(&mut self, id: LayerID) -> LayerHandle<'_> {
        self.document.ensure_exists(id);
        LayerHandle {
            sketch: self,
            layer_id: id,
        }
    }

    // =========================================================================
    // Stroke/fill layer routing
    // =========================================================================

    /// Set which layer receives stroke (outline). `None` disables stroke.
    ///
    /// Default: `Some(0)`.
    ///
    /// # Example
    /// ```
    /// # use whiskers::prelude::*;
    /// # let mut sketch = Sketch::new();
    /// sketch
    ///     .stroke_layer(Some(0))   // strokes go to layer 0
    ///     .fill_layer(Some(1))     // fills go to layer 1
    ///     .circle(50.0, 50.0, 25.0);
    /// ```
    pub fn stroke_layer(&mut self, layer: Option<LayerID>) -> &mut Self {
        if let Some(id) = layer {
            self.document.ensure_exists(id);
        }
        self.stroke_layer = layer;
        self
    }

    /// Set which layer receives fill (hatching). `None` disables fill.
    ///
    /// Default: `None`.
    ///
    /// When enabled, closed shapes will be hatched using the fill layer's
    /// `pen_width` as spacing and `hatch_angle` for orientation.
    pub fn fill_layer(&mut self, layer: Option<LayerID>) -> &mut Self {
        if let Some(id) = layer {
            self.document.ensure_exists(id);
        }
        self.fill_layer = layer;
        self
    }

    /// Shorthand: enable stroke on given layer, disable fill.
    pub fn stroke_only(&mut self, layer: LayerID) -> &mut Self {
        self.stroke_layer(Some(layer)).fill_layer(None)
    }

    /// Shorthand: enable fill on given layer, disable stroke.
    pub fn fill_only(&mut self, layer: LayerID) -> &mut Self {
        self.stroke_layer(None).fill_layer(Some(layer))
    }

    /// Shorthand: enable both stroke and fill on same layer.
    pub fn stroke_and_fill(&mut self, layer: LayerID) -> &mut Self {
        self.stroke_layer(Some(layer)).fill_layer(Some(layer))
    }

    // =========================================================================
    // Style stack
    // =========================================================================

    /// Push the current style state onto the stack.
    ///
    /// Saves: `stroke_layer`, `fill_layer`, `color`, `stroke_width`.
    /// Restore with [`pop_style`](Self::pop_style).
    ///
    /// # Example
    /// ```
    /// # use whiskers::prelude::*;
    /// # let mut sketch = Sketch::new();
    /// sketch
    ///     .stroke_layer(Some(0))
    ///     .push_style()
    ///     .stroke_layer(None)  // temporarily disable stroke
    ///     .fill_layer(Some(1))
    ///     .circle(50.0, 50.0, 25.0)  // fill only
    ///     .pop_style()
    ///     .circle(100.0, 50.0, 25.0); // stroke restored
    /// ```
    pub fn push_style(&mut self) -> &mut Self {
        self.style_stack.push(StyleState {
            stroke_layer: self.stroke_layer,
            fill_layer: self.fill_layer,
            path_metadata: self.path_metadata.clone(),
        });
        self
    }

    /// Pop the style state from the stack, restoring previous settings.
    pub fn pop_style(&mut self) -> &mut Self {
        if let Some(state) = self.style_stack.pop() {
            self.stroke_layer = state.stroke_layer;
            self.fill_layer = state.fill_layer;
            self.path_metadata = state.path_metadata;
        } else {
            log::warn!("pop_style: stack underflow");
        }
        self
    }

    /// Push style, apply closure, pop style.
    ///
    /// Convenience method for temporary style changes.
    ///
    /// # Example
    /// ```
    /// # use whiskers::prelude::*;
    /// # let mut sketch = Sketch::new();
    /// sketch
    ///     .circle(50.0, 50.0, 25.0)  // default style
    ///     .with_style(|s| {
    ///         s.color(Color::RED)
    ///          .circle(100.0, 50.0, 25.0);  // red
    ///     })
    ///     .circle(150.0, 50.0, 25.0); // back to default
    /// ```
    pub fn with_style(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.push_style();
        f(self);
        self.pop_style();
        self
    }

    // =========================================================================
    // Transform stack
    // =========================================================================

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

    /// Consume the [`Sketch`] and return the underlying [`Document`].
    pub fn into_document(self) -> Document {
        self.document
    }

    /// Opens the `vsvg` viewer with the sketch content.
    ///
    /// Requires the `viewer` feature to be enabled.
    #[cfg(feature = "viewer")]
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::missing_panics_doc)]
    pub fn show(&mut self) -> anyhow::Result<&mut Self> {
        use std::mem::{replace, take};
        use std::sync::Arc;

        let document = Arc::new(take(&mut self.document));
        vsvg_viewer::show(document.clone())?;

        let _ = replace(
            &mut self.document,
            Arc::into_inner(document)
                .expect("vsvg_viewer::show does not keep references to the document"),
        );
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

        // Route to stroke layer
        if let Some(layer_id) = self.stroke_layer {
            self.document.push_path(layer_id, path.clone());
        }

        // Route to fill layer (hatching)
        if let Some(fill_layer_id) = self.fill_layer {
            // Only hatch closed paths
            if path.data().is_closed() {
                // Get spacing from layer's pen_width
                let spacing = self
                    .document
                    .try_get(fill_layer_id)
                    .and_then(|layer| layer.metadata().default_path_metadata.stroke_width)
                    .unwrap_or(1.0);

                // Get hatch angle from sketch state
                let angle = self
                    .hatch_angles
                    .get(&fill_layer_id)
                    .copied()
                    .unwrap_or(0.0);

                let params = vsvg::HatchParams::new(spacing).with_angle(angle);

                if let Ok(hatch_paths) = path.hatch(&params, self.tolerance) {
                    for hatch_path in hatch_paths {
                        self.document
                            .push_path(fill_layer_id, Path::from(hatch_path));
                    }
                }
            }
        }

        self
    }
}

// =============================================================================
// LayerHandle
// =============================================================================

/// Handle for configuring layer properties.
///
/// Obtained via [`Sketch::layer`]. Changes are applied immediately to the
/// layer's metadata.
///
/// # Example
/// ```
/// # use whiskers::prelude::*;
/// # let mut sketch = Sketch::new();
/// sketch
///     .layer(0).pen_width(0.5 * Unit::Mm).color(Color::BLACK)
///     .layer(1).pen_width(0.3 * Unit::Mm).hatch_angle(std::f64::consts::FRAC_PI_4);
/// ```
pub struct LayerHandle<'a> {
    sketch: &'a mut Sketch,
    layer_id: LayerID,
}

#[expect(
    clippy::return_self_not_must_use,
    reason = "methods have side effects; return is for chaining"
)]
impl<'a> LayerHandle<'a> {
    /// Set the pen width for this layer.
    ///
    /// This sets the layer's `default_path_metadata.stroke_width`.
    /// When this layer is used as a fill layer, `pen_width` controls hatch spacing.
    ///
    /// Accepts any type that converts to [`Length`], including:
    /// - `f64` (raw pixels)
    /// - `0.5 * Unit::Mm` (millimeters)
    /// - `Length::new(0.3, Unit::Cm)` (centimeters)
    pub fn pen_width(self, width: impl Into<Length>) -> Self {
        let width_px: f64 = width.into().into();
        self.sketch
            .document
            .get_mut(self.layer_id)
            .metadata_mut()
            .default_path_metadata
            .stroke_width = Some(width_px);
        self
    }

    /// Set the default color for this layer.
    pub fn color(self, color: impl Into<vsvg::Color>) -> Self {
        self.sketch
            .document
            .get_mut(self.layer_id)
            .metadata_mut()
            .default_path_metadata
            .color = Some(color.into());
        self
    }

    /// Set the layer name.
    pub fn name(self, name: impl Into<String>) -> Self {
        self.sketch
            .document
            .get_mut(self.layer_id)
            .metadata_mut()
            .name = Some(name.into());
        self
    }

    /// Set the hatch angle for this layer (in radians).
    ///
    /// When this layer is used as a fill layer, closed shapes will be
    /// hatched at this angle. Default: 0 (horizontal).
    pub fn hatch_angle(self, angle: f64) -> Self {
        self.sketch.hatch_angles.insert(self.layer_id, angle);
        self
    }

    /// Start configuring a different layer (chaining).
    pub fn layer(self, id: LayerID) -> LayerHandle<'a> {
        self.sketch.layer(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vsvg::{Color, Draw, Unit};

    #[test]
    fn test_layer_pen_width() {
        let mut sketch = Sketch::new();
        sketch.layer(0).pen_width(0.5 * Unit::Mm);

        let layer = sketch.document().try_get(0).unwrap();
        let expected = 0.5 * 96.0 / 25.4; // mm to pixels
        let actual = layer.metadata().default_path_metadata.stroke_width.unwrap();
        assert!((actual - expected).abs() < 0.001);
    }

    #[test]
    fn test_layer_color() {
        let mut sketch = Sketch::new();
        sketch.layer(0).color(Color::RED);

        let layer = sketch.document().try_get(0).unwrap();
        assert_eq!(
            layer.metadata().default_path_metadata.color,
            Some(Color::RED)
        );
    }

    #[test]
    fn test_layer_hatch_angle() {
        let mut sketch = Sketch::new();
        sketch.layer(1).hatch_angle(std::f64::consts::FRAC_PI_4);

        let angle = sketch.hatch_angles.get(&1).copied().unwrap();
        assert!((angle - std::f64::consts::FRAC_PI_4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_layer_chaining() {
        let mut sketch = Sketch::new();
        sketch
            .layer(0)
            .pen_width(0.5 * Unit::Mm)
            .color(Color::BLACK)
            .layer(1)
            .pen_width(0.3 * Unit::Mm)
            .color(Color::RED);

        let layer0 = sketch.document().try_get(0).unwrap();
        let layer1 = sketch.document().try_get(1).unwrap();

        assert_eq!(
            layer0.metadata().default_path_metadata.color,
            Some(Color::BLACK)
        );
        assert_eq!(
            layer1.metadata().default_path_metadata.color,
            Some(Color::RED)
        );
    }

    #[test]
    fn test_stroke_layer_routing() {
        let mut sketch = Sketch::new();
        sketch.stroke_layer(Some(0)).fill_layer(None);
        sketch.circle(50.0, 50.0, 25.0);

        assert_eq!(sketch.document().try_get(0).unwrap().paths().len(), 1);
    }

    #[test]
    fn test_stroke_only() {
        let mut sketch = Sketch::new();
        sketch.stroke_only(1);

        assert_eq!(sketch.stroke_layer, Some(1));
        assert_eq!(sketch.fill_layer, None);
    }

    #[test]
    fn test_fill_only() {
        let mut sketch = Sketch::new();
        sketch.fill_only(1);

        assert_eq!(sketch.stroke_layer, None);
        assert_eq!(sketch.fill_layer, Some(1));
    }

    #[test]
    fn test_stroke_and_fill() {
        let mut sketch = Sketch::new();
        sketch.stroke_and_fill(2);

        assert_eq!(sketch.stroke_layer, Some(2));
        assert_eq!(sketch.fill_layer, Some(2));
    }

    #[test]
    fn test_push_pop_style() {
        let mut sketch = Sketch::new();
        sketch.stroke_layer(Some(0)).color(Color::BLACK);

        sketch.push_style();
        sketch.stroke_layer(Some(1)).color(Color::RED);

        assert_eq!(sketch.stroke_layer, Some(1));

        sketch.pop_style();

        assert_eq!(sketch.stroke_layer, Some(0));
        assert_eq!(sketch.path_metadata.color, Some(Color::BLACK));
    }

    #[test]
    fn test_with_style() {
        let mut sketch = Sketch::new();
        sketch.stroke_layer(Some(0));

        sketch.with_style(|s| {
            s.stroke_layer(Some(1));
            assert_eq!(s.stroke_layer, Some(1));
        });

        assert_eq!(sketch.stroke_layer, Some(0));
    }

    #[test]
    fn test_style_stack_underflow_warning() {
        let mut sketch = Sketch::new();
        // Should warn but not panic
        sketch.pop_style();
    }

    #[test]
    fn test_fill_layer_hatching() {
        let mut sketch = Sketch::new();
        sketch
            .layer(1)
            .pen_width(5.0) // Large spacing for visible hatching
            .hatch_angle(0.0);
        sketch.stroke_layer(None).fill_layer(Some(1));

        // Draw a closed rectangle - should produce hatch lines
        sketch.rect(0.0, 0.0, 50.0, 50.0);

        let layer1 = sketch.document().try_get(1).unwrap();
        // Should have at least one hatch line
        assert!(
            !layer1.paths().is_empty(),
            "fill layer should have hatch paths"
        );
    }

    #[test]
    fn test_open_path_not_hatched() {
        let mut sketch = Sketch::new();
        sketch.layer(1).pen_width(2.0);
        sketch.stroke_layer(None).fill_layer(Some(1));

        // Draw an open line - should NOT produce hatch lines
        sketch.line(0.0, 0.0, 100.0, 100.0);

        let layer1 = sketch.document().try_get(1).unwrap();
        assert!(
            layer1.paths().is_empty(),
            "open paths should not be hatched"
        );
    }
}
