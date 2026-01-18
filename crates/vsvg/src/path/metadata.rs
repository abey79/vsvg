use crate::{Color, Length};

/// Metadata for a path (color, stroke width).
///
/// Fields are `Option` to support inheritance from layer defaults.
/// `None` means "inherit from layer" (or SVG default if layer also `None`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PathMetadata {
    /// Path color. `None` means inherit from layer.
    pub color: Option<Color>,

    /// Stroke width in pixels (96 DPI). `None` means inherit from layer.
    ///
    /// SVG default is 1.0 pixel.
    pub stroke_width: Option<f64>,
}

impl PathMetadata {
    /// Set color, returning self for chaining.
    #[must_use]
    pub fn with_color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set stroke width, returning self for chaining.
    ///
    /// Accepts `f64` (pixels) or `Length` (e.g., `0.5 * Unit::Mm`).
    #[must_use]
    pub fn with_stroke_width(mut self, width: impl Into<Length>) -> Self {
        let length: Length = width.into();
        self.stroke_width = Some(length.into());
        self
    }

    /// Resolve this metadata against layer defaults.
    ///
    /// Returns concrete values for display purposes (e.g., vsvg-viewer).
    /// Path-level values override layer-level; fallback to SVG defaults.
    ///
    /// # SVG Defaults
    /// - `color`: black (#000000)
    /// - `stroke_width`: 1.0 pixel
    #[must_use]
    pub fn resolve(&self, layer_defaults: &PathMetadata) -> ResolvedPathMetadata {
        ResolvedPathMetadata {
            color: self.color.or(layer_defaults.color).unwrap_or(Color::BLACK),
            stroke_width: self
                .stroke_width
                .or(layer_defaults.stroke_width)
                .unwrap_or(1.0),
        }
    }

    /// Merge another metadata into this one.
    ///
    /// For each field:
    /// - If only one is `Some`, use that value.
    /// - If both are `Some` and equal, keep the value.
    /// - If both are `Some` but different, result is `None` (conflict).
    pub fn merge(&mut self, other: &PathMetadata) {
        self.color = match (self.color, other.color) {
            (Some(a), None) | (None, Some(a)) => Some(a),
            (Some(a), Some(b)) if a == b => Some(a),
            (Some(_), Some(_)) | (None, None) => None, // Conflict or both None
        };
        self.stroke_width = match (self.stroke_width, other.stroke_width) {
            (Some(a), None) | (None, Some(a)) => Some(a),
            (Some(a), Some(b)) if (a - b).abs() < f64::EPSILON => Some(a),
            (Some(_), Some(_)) | (None, None) => None, // Conflict or both None
        };
    }

    /// Returns true if all fields are `None`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.color.is_none() && self.stroke_width.is_none()
    }
}

/// Fully resolved path metadata with concrete values.
///
/// Used for display (e.g., vsvg-viewer) where concrete values are needed.
/// Obtained by calling [`PathMetadata::resolve`] with layer defaults.
///
/// # SVG Defaults
/// These match SVG specification defaults:
/// - `color`: black (#000000)
/// - `stroke_width`: 1.0 pixel
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPathMetadata {
    pub color: Color,
    pub stroke_width: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path_overrides_layer() {
        let path_meta = PathMetadata::default().with_color(Color::RED);
        let layer_defaults = PathMetadata::default()
            .with_color(Color::BLUE)
            .with_stroke_width(2.0);

        let resolved = path_meta.resolve(&layer_defaults);

        assert_eq!(resolved.color, Color::RED); // path wins
        assert_eq!(resolved.stroke_width, 2.0); // layer default
    }

    #[test]
    fn test_resolve_fallback_to_svg_defaults() {
        let path_meta = PathMetadata::default();
        let layer_defaults = PathMetadata::default();

        let resolved = path_meta.resolve(&layer_defaults);

        assert_eq!(resolved.color, Color::BLACK); // SVG default
        assert_eq!(resolved.stroke_width, 1.0); // SVG default
    }

    #[test]
    fn test_merge_conflict_becomes_none() {
        let mut meta1 = PathMetadata::default().with_color(Color::RED);
        let meta2 = PathMetadata::default().with_color(Color::BLUE);

        meta1.merge(&meta2);

        assert_eq!(meta1.color, None); // Conflict: RED != BLUE -> None
    }

    #[test]
    fn test_merge_same_value_kept() {
        let mut meta1 = PathMetadata::default().with_color(Color::RED);
        let meta2 = PathMetadata::default().with_color(Color::RED);

        meta1.merge(&meta2);

        assert_eq!(meta1.color, Some(Color::RED)); // Same value kept
    }

    #[test]
    fn test_merge_none_takes_other() {
        let mut meta1 = PathMetadata::default();
        let meta2 = PathMetadata::default().with_color(Color::BLUE);

        meta1.merge(&meta2);

        assert_eq!(meta1.color, Some(Color::BLUE)); // None takes other
    }

    #[test]
    fn test_is_empty() {
        assert!(PathMetadata::default().is_empty());
        assert!(!PathMetadata::default().with_color(Color::RED).is_empty());
        assert!(!PathMetadata::default().with_stroke_width(2.0).is_empty());
    }
}
