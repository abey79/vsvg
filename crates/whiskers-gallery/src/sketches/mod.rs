//! Gallery sketch modules.

pub mod hello_world;
pub mod schotter;

/// Metadata for gallery display and HTML generation.
#[derive(Debug, Clone)]
pub struct SketchMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub author: &'static str,
}
