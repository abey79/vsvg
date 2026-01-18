use crate::PathMetadata;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LayerMetadata {
    pub name: Option<String>,

    /// Default metadata for paths in this layer.
    ///
    /// Paths inherit these values when their own fields are `None`.
    /// Written to the SVG `<g>` element for the layer.
    pub default_path_metadata: PathMetadata,
}

impl LayerMetadata {
    /// Merge with another [`LayerMetadata`].
    ///
    /// Only the common attributes are kept, the others are discarded.
    pub fn merge(&mut self, other: &Self) {
        if self.name != other.name {
            self.name = None;
        }
        self.default_path_metadata
            .merge(&other.default_path_metadata);
    }
}
