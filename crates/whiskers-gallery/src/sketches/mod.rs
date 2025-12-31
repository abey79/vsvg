//! Gallery sketch registry and metadata.

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

/// Registry of all available sketches.
pub static SKETCH_MANIFEST: &[SketchMeta] = &[
    SketchMeta {
        id: "schotter",
        name: "Schotter",
        description: "Recreation of Georg Nees' classic 1968-1970 generative art piece",
        author: "Antoine Beyeler",
    },
    SketchMeta {
        id: "hello_world",
        name: "Hello World",
        description: "A simple introductory sketch demonstrating basic whiskers usage",
        author: "Antoine Beyeler",
    },
];
