use crate::PageSize;

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentMetadata {
    pub page_size: Option<PageSize>,
    pub source: Option<String>,

    /// Whether to include the current date in SVG output. Defaults to `true`.
    /// Set to `false` for deterministic/reproducible SVG output.
    pub include_date: bool,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            page_size: None,
            source: None,
            include_date: true,
        }
    }
}

impl DocumentMetadata {
    pub(super) fn with_source_suffix(&self, suffix: &str) -> Self {
        Self {
            source: Some(format!(
                "{}{}",
                self.source.as_deref().unwrap_or("<empty>"),
                suffix
            )),
            ..self.clone()
        }
    }
}
