use crate::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct PathMetadata {
    pub color: Color,
    pub stroke_width: f64,
}

impl Default for PathMetadata {
    fn default() -> Self {
        Self {
            stroke_width: 1.0,
            color: Color::default(),
        }
    }
}

impl PathMetadata {
    /// Merge another metadata into this one.
    ///
    /// Currently: the first one wins (self-unchanged).
    //TODO: metadata should probably have `Option`, so the merge can be smart.
    pub fn merge(&mut self, _other: &PathMetadata) {
        // For now, keep self's values (first path wins)
        // Future enhancement: if self.color.is_none() { self.color = other.color; }
    }
}
