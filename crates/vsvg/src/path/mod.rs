mod flattened_path;
pub(crate) mod into_bezpath;
mod metadata;
#[allow(clippy::module_inception)]
mod path;
mod point;

use crate::{SvgPathWriter, Transforms};

pub use flattened_path::{
    FlattenedPath, Polyline, multi_polygon_to_flattened_paths, polygon_to_flattened_paths,
};
pub use into_bezpath::{IntoBezPath, IntoBezPathTolerance};
pub use metadata::{PathMetadata, ResolvedPathMetadata};
pub use path::Path;
pub use point::Point;

/// Error type for path-to-polygon conversion.
#[derive(Debug, Clone, PartialEq)]
pub enum ToGeoPolygonError {
    /// The exterior ring (first subpath) is not closed.
    ExteriorNotClosed,

    /// The exterior ring has fewer than 3 distinct points.
    ExteriorTooFewPoints,

    /// An interior ring (hole) is not closed. Contains the ring index (0-based).
    InteriorNotClosed(usize),

    /// An interior ring (hole) has fewer than 3 points. Contains the ring index (0-based).
    InteriorTooFewPoints(usize),

    /// The path is empty (no subpaths).
    EmptyPath,

    /// The resulting polygon failed geo validation.
    /// Wraps geo's `InvalidPolygon` for details (self-intersection, etc.)
    InvalidPolygon(geo::algorithm::validation::InvalidPolygon),
}

impl std::fmt::Display for ToGeoPolygonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExteriorNotClosed => write!(f, "exterior ring is not closed"),
            Self::ExteriorTooFewPoints => write!(f, "exterior ring has fewer than 3 points"),
            Self::InteriorNotClosed(i) => write!(f, "interior ring {i} is not closed"),
            Self::InteriorTooFewPoints(i) => {
                write!(f, "interior ring {i} has fewer than 3 points")
            }
            Self::EmptyPath => write!(f, "path is empty"),
            Self::InvalidPolygon(e) => write!(f, "invalid polygon: {e:?}"),
        }
    }
}

impl std::error::Error for ToGeoPolygonError {}

impl From<geo::algorithm::validation::InvalidPolygon> for ToGeoPolygonError {
    fn from(e: geo::algorithm::validation::InvalidPolygon) -> Self {
        Self::InvalidPolygon(e)
    }
}

pub const DEFAULT_TOLERANCE: f64 = 0.05;

pub trait PathDataTrait:
    Transforms + SvgPathWriter + Default + Clone + PartialEq + std::fmt::Debug
{
    fn bounds(&self) -> kurbo::Rect;
    fn start(&self) -> Option<Point>;
    fn end(&self) -> Option<Point>;
    fn is_point(&self) -> bool;
    fn flip(&mut self);

    /// Returns true if the path is closed (start ≈ end within [`crate::SAME_POINT_EPSILON`]).
    fn is_closed(&self) -> bool {
        match (self.start(), self.end()) {
            (Some(start), Some(end)) => start.distance(&end) < crate::SAME_POINT_EPSILON,
            _ => false,
        }
    }
}

pub trait PathTrait<D: PathDataTrait>: Transforms + Clone + PartialEq + std::fmt::Debug {
    fn data(&self) -> &D;

    fn data_mut(&mut self) -> &mut D;

    fn into_data(self) -> D;

    fn bounds(&self) -> kurbo::Rect;

    fn start(&self) -> Option<Point> {
        self.data().start()
    }

    fn end(&self) -> Option<Point> {
        self.data().end()
    }

    fn metadata(&self) -> &PathMetadata;
    fn metadata_mut(&mut self) -> &mut PathMetadata;

    /// Append another path to this one.
    ///
    /// If the endpoint of `self` and the start of `other` are within `epsilon`, the duplicate point
    /// is skipped (for polylines) or `MoveTo` is converted to `LineTo` (for `BezPath`s) to create a
    /// continuous path.
    ///
    /// Metadata is merged (currently first path's metadata wins).
    fn join(&mut self, other: &Self, epsilon: f64);

    /// Split a compound path into its individual subpaths.
    ///
    /// For `BezPath`-based paths, each `MoveTo` element starts a new subpath. For polyline-based
    /// paths, this returns the path unchanged (polylines cannot represent compound paths).
    ///
    /// Metadata is cloned to all resulting paths.
    fn split(self) -> Vec<Self>;
}
