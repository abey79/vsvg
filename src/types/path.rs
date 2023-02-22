use crate::types::crop::Crop;
use crate::types::{Color, Polylines};
use kurbo::BezPath;

const DEFAULT_TOLERANCE: f64 = 0.1;

#[derive(Clone, Debug)]
pub struct Path {
    pub bezpath: BezPath,
    pub color: Color,
    pub stroke_width: f64,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            bezpath: BezPath::new(),
            color: Color::default(),
            stroke_width: 1.0,
        }
    }
}

impl Path {
    #[allow(dead_code)]
    pub fn from_shape<T: kurbo::Shape>(path: T) -> Self {
        Self::from_shape_with_tolerance(path, DEFAULT_TOLERANCE)
    }

    #[allow(dead_code)]
    pub fn from_shape_with_tolerance<T: kurbo::Shape>(path: T, tolerance: f64) -> Self {
        Self {
            bezpath: path.into_path(tolerance),
            ..Default::default()
        }
    }

    pub fn flatten(&self, tolerance: f64) -> Polylines {
        Polylines::from_path(self, tolerance)
    }

    pub fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        let new_bezpath =
            BezPath::from_path_segments(self.bezpath.segments().flat_map(|segment| {
                match segment {
                    kurbo::PathSeg::Line(line) => line
                        .crop(x_min, y_min, x_max, y_max)
                        .into_iter()
                        .map(kurbo::PathSeg::Line)
                        .collect(),
                    kurbo::PathSeg::Cubic(cubic) => cubic
                        .crop(x_min, y_min, x_max, y_max)
                        .into_iter()
                        .map(kurbo::PathSeg::Cubic)
                        .collect(),
                    kurbo::PathSeg::Quad(_) => vec![], // TODO: implement for completeness
                }
            }));

        Self {
            bezpath: new_bezpath,
            ..self
        }
    }
}
