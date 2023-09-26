use crate::PageSize;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocumentMetadata {
    pub page_size: Option<PageSize>,
    pub source: Option<String>,
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
