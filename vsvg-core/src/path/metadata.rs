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
