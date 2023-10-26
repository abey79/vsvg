use crate::Sketch;

/// Trait to share a build method that calls current sketch reference between
/// the grid helpers
pub trait GridBuild<CellType> {
    /// Computes grid's cell data such as coordinates (column and row),
    /// size and canvas position. See `grid.rs` and `hex_grid.rs` in examples.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn build(self, sketch: &mut Sketch, callback_fn: impl FnOnce(&mut Sketch, &CellType) + Copy);
}
