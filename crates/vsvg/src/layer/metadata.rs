#[derive(Debug, Clone, Default, PartialEq)]
pub struct LayerMetadata {
    pub name: Option<String>,
    //TODO(#4): add default path metadata
}

impl LayerMetadata {
    /// Merge with another [`LayerMetadata`].
    ///
    /// Only the common attributes are kept, the other are discarded.
    pub fn merge(&mut self, other: &Self) {
        if self.name != other.name {
            self.name = None;
        }
    }
}
