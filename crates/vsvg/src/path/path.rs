use super::{
    FlattenedPath, PathDataTrait, PathMetadata, PathTrait, Point, Polyline,
    multi_polygon_to_flattened_paths,
};
use crate::Transforms;
use crate::crop::{Crop, QuadCropResult, crop_quad_bezier};
use crate::path::into_bezpath::{
    IntoBezPath, IntoBezPathTolerance, line_segment_to_bezpath, points_to_bezpath,
};
use kurbo::{Affine, BezPath, PathEl};
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;

// ======================================================================================
// The path data for `Path` is `kurbo::BezPath`.

impl Transforms for BezPath {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        self.apply_affine(*affine);
        self
    }
}

impl PathDataTrait for BezPath {
    fn bounds(&self) -> kurbo::Rect {
        kurbo::Shape::bounding_box(self)
    }

    fn start(&self) -> Option<Point> {
        if let Some(PathEl::MoveTo(pt)) = self.elements().first() {
            Some(pt.into())
        } else {
            None
        }
    }

    fn end(&self) -> Option<Point> {
        match self.elements().last()? {
            PathEl::MoveTo(pt)
            | PathEl::LineTo(pt)
            | PathEl::QuadTo(_, pt)
            | PathEl::CurveTo(_, _, pt) => Some(pt.into()),
            PathEl::ClosePath => {
                // since this may be a compound path, we must search backwards
                for el in self.elements().iter().rev() {
                    if let PathEl::MoveTo(pt) = el {
                        return Some(pt.into());
                    }
                }

                None
            }
        }
    }

    fn is_point(&self) -> bool {
        matches!(self.elements(), [PathEl::MoveTo(a), PathEl::LineTo(b)] if a == b)
    }

    fn flip(&mut self) {
        let segs: Vec<kurbo::PathSeg> = self.segments().collect();
        *self = BezPath::from_path_segments(segs.into_iter().rev().map(|seg| seg.reverse()));
    }
}

// ======================================================================================
// `Path`

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    pub data: BezPath,
    pub(crate) metadata: PathMetadata,
}

impl Transforms for Path {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        self.data.apply_affine(*affine);
        self
    }
}

impl PathTrait<BezPath> for Path {
    fn data(&self) -> &BezPath {
        &self.data
    }

    fn data_mut(&mut self) -> &mut BezPath {
        &mut self.data
    }

    fn into_data(self) -> BezPath {
        self.data
    }

    fn bounds(&self) -> kurbo::Rect {
        self.data.bounds()
    }
    fn metadata(&self) -> &PathMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut PathMetadata {
        &mut self.metadata
    }

    /// Append another path to this one.
    ///
    /// If the endpoint of `self` and the start of `other` are within `epsilon`,
    /// the initial `MoveTo` of `other` is converted to `LineTo` to create a continuous path.
    /// Otherwise, the `MoveTo` is kept, creating a compound path with multiple subpaths.
    ///
    /// Metadata is merged (currently first path's metadata wins).
    fn join(&mut self, other: &Path, epsilon: f64) {
        let should_connect = match (self.data.end(), other.data.start()) {
            (Some(end), Some(start)) => end.distance(&start) < epsilon,
            _ => false,
        };

        for (i, el) in other.data.elements().iter().enumerate() {
            if i == 0 {
                match el {
                    PathEl::MoveTo(pt) if should_connect => {
                        self.data.push(PathEl::LineTo(*pt));
                    }
                    _ => self.data.push(*el),
                }
            } else {
                self.data.push(*el);
            }
        }

        self.metadata.merge(&other.metadata);
    }

    /// Split a compound path into its individual paths.
    ///
    /// The returned paths always consist of a single, non-compound path.
    fn split(self) -> Vec<Self> {
        let elements = self.data.elements();
        if elements.is_empty() {
            return vec![];
        }

        let mut result = Vec::new();
        let mut current = BezPath::new();

        for el in elements {
            match el {
                PathEl::MoveTo(pt) => {
                    // Save current path if non-empty and start new one
                    if !current.elements().is_empty() {
                        result.push(Path {
                            data: std::mem::take(&mut current),
                            metadata: self.metadata.clone(),
                        });
                    }
                    current.push(PathEl::MoveTo(*pt));
                }
                _ => {
                    current.push(*el);
                }
            }
        }

        // Remember the last subpath
        if !current.elements().is_empty() {
            result.push(Path {
                data: current,
                metadata: self.metadata,
            });
        }

        result
    }
}

impl Path {
    #[must_use]
    pub fn from_tolerance(path: impl IntoBezPathTolerance, tolerance: f64) -> Self {
        Self {
            data: path.into_bezpath_with_tolerance(tolerance),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn from_metadata(path: impl IntoBezPath, metadata: PathMetadata) -> Self {
        Self {
            data: path.into_bezpath(),
            metadata,
        }
    }

    #[must_use]
    pub fn from_tolerance_metadata(
        path: impl IntoBezPathTolerance,
        tolerance: f64,
        metadata: PathMetadata,
    ) -> Self {
        Self {
            data: path.into_bezpath_with_tolerance(tolerance),
            metadata,
        }
    }

    #[must_use]
    pub fn from_points(points: impl IntoIterator<Item = impl Into<Point>>) -> Self {
        Self::from(points_to_bezpath(points))
    }

    #[must_use]
    pub fn from_segments(points: impl IntoIterator<Item = impl Into<(Point, Point)>>) -> Self {
        Self::from(line_segment_to_bezpath(points))
    }

    pub fn from_svg(path: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            data: BezPath::from_svg(path)?,
            ..Default::default()
        })
    }

    pub fn apply_transform(&mut self, transform: Affine) {
        self.data.apply_affine(transform);
    }

    #[must_use]
    pub fn flatten(&self, tolerance: f64) -> Vec<FlattenedPath> {
        crate::trace_function!();

        let mut lines: Vec<FlattenedPath> = vec![];
        let current_line: RefCell<Polyline> = RefCell::new(Polyline::default());

        kurbo::flatten(&self.data, tolerance, |el| match el {
            PathEl::MoveTo(pt) => {
                if !current_line.borrow().points().is_empty() {
                    // lines.push(Polyline::from(current_line.replace(vec![])));
                    lines.push(FlattenedPath::from(
                        current_line.replace(Polyline::default()),
                    ));
                }
                current_line.borrow_mut().points_mut().push(pt.into());
            }
            PathEl::LineTo(pt) => current_line.borrow_mut().points_mut().push(pt.into()),
            PathEl::ClosePath => {
                let pt = current_line.borrow().points()[0];
                current_line.borrow_mut().points_mut().push(pt);
            }
            _ => unreachable!(),
        });

        let current_line = current_line.into_inner();
        if !current_line.points().is_empty() {
            lines.push(FlattenedPath::from(current_line));
        }

        for line in &mut lines {
            *line.metadata_mut() = self.metadata().clone();
        }

        lines
    }

    #[must_use]
    pub fn bezier_handles(&self) -> Vec<FlattenedPath> {
        crate::trace_function!();

        self.data
            .segments()
            .filter_map(|segment| match segment {
                kurbo::PathSeg::Cubic(cubic) => Some([
                    vec![cubic.p0.into(), cubic.p1.into()],
                    vec![cubic.p2.into(), cubic.p3.into()],
                ]),
                kurbo::PathSeg::Quad(quad) => Some([
                    vec![quad.p0.into(), quad.p1.into()],
                    vec![quad.p1.into(), quad.p2.into()],
                ]),
                kurbo::PathSeg::Line(_) => None,
            })
            .flatten()
            .map(FlattenedPath::from)
            .collect()
    }

    pub fn crop(
        &mut self,
        x_min: impl Into<f64>,
        y_min: impl Into<f64>,
        x_max: impl Into<f64>,
        y_max: impl Into<f64>,
    ) -> &Self {
        crate::trace_function!();

        let x_min = x_min.into();
        let y_min = y_min.into();
        let x_max = x_max.into();
        let y_max = y_max.into();

        let new_bezpath = BezPath::from_path_segments(self.data.segments().flat_map(|segment| {
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
                kurbo::PathSeg::Quad(quad) => {
                    match crop_quad_bezier(quad, x_min, y_min, x_max, y_max) {
                        QuadCropResult::Quad(quad) => vec![kurbo::PathSeg::Quad(quad)],
                        QuadCropResult::Cubic(cubic) => {
                            cubic.into_iter().map(kurbo::PathSeg::Cubic).collect()
                        }
                    }
                }
            }
        }));

        self.data = new_bezpath;
        self
    }

    /// Convert path to `geo::Polygon`, flattening curves with given tolerance.
    ///
    /// For compound paths (multiple subpaths):
    /// - First subpath becomes the exterior ring
    /// - Subsequent closed subpaths become interior rings (holes)
    /// - Unclosed interior subpaths return an error
    ///
    /// # Errors
    ///
    /// See [`super::ToGeoPolygonError`]
    pub fn to_geo_polygon(
        &self,
        tolerance: f64,
    ) -> Result<geo::Polygon<f64>, super::ToGeoPolygonError> {
        use super::ToGeoPolygonError;
        use geo::algorithm::validation::Validation;

        // Flatten curves to polylines (one per subpath)
        let flattened = self.flatten(tolerance);

        let mut iter = flattened.into_iter();

        // First subpath is exterior ring
        let Some(exterior_polyline) = iter.next() else {
            return Err(ToGeoPolygonError::EmptyPath);
        };
        let exterior_points = exterior_polyline.data.points();

        if exterior_points.len() < 3 {
            return Err(ToGeoPolygonError::ExteriorTooFewPoints);
        }

        if !exterior_polyline.data.is_closed() {
            return Err(ToGeoPolygonError::ExteriorNotClosed);
        }

        let exterior_coords: Vec<geo::Coord<f64>> = exterior_points
            .iter()
            .map(|p| geo::Coord { x: p.x(), y: p.y() })
            .collect();
        let exterior = geo::LineString::new(exterior_coords);

        // Remaining subpaths are holes (interior rings)
        let mut interiors = Vec::new();
        for (i, hole_path) in iter.enumerate() {
            let hole_points = hole_path.data.points();

            if hole_points.len() < 3 {
                return Err(ToGeoPolygonError::InteriorTooFewPoints(i));
            }

            if !hole_path.data.is_closed() {
                return Err(ToGeoPolygonError::InteriorNotClosed(i));
            }

            let hole_coords: Vec<geo::Coord<f64>> = hole_points
                .iter()
                .map(|p| geo::Coord { x: p.x(), y: p.y() })
                .collect();
            interiors.push(geo::LineString::new(hole_coords));
        }

        let polygon = geo::Polygon::new(exterior, interiors);

        // Validate the resulting polygon (checks self-intersection, etc.)
        polygon.check_validation()?;

        Ok(polygon)
    }

    /// Buffer (expand or shrink) this closed path.
    ///
    /// - Positive distance = expand outward
    /// - Negative distance = shrink inward
    ///
    /// Returns flattened paths since buffer operations work on polygons.
    /// May return multiple paths if the shape splits, or empty if fully eroded.
    ///
    /// # Errors
    /// Returns error if path cannot be converted to polygon (not closed, etc.)
    ///
    /// # Example
    /// ```
    /// use vsvg::{Path, Unit};
    /// use kurbo::Circle;
    ///
    /// let circle = Path::from(Circle::new((0.0, 0.0), 10.0));
    ///
    /// // Expand by 1 unit
    /// let expanded = circle.buffer(1.0, 0.1).unwrap();
    ///
    /// // Shrink by 0.5 units (for hatching inset)
    /// let shrunk = circle.buffer(-0.5, 0.1).unwrap();
    /// ```
    pub fn buffer(
        &self,
        distance: impl Into<crate::Length>,
        tolerance: f64,
    ) -> Result<Vec<FlattenedPath>, super::ToGeoPolygonError> {
        use geo::Buffer;

        let polygon = self.to_geo_polygon(tolerance)?;
        let distance_f64: f64 = distance.into().into();
        let multi_polygon = polygon.buffer(distance_f64);

        Ok(multi_polygon_to_flattened_paths(&multi_polygon))
    }

    /// Generate hatching (boundary + fill lines) for this closed path.
    ///
    /// Returns a `Vec<FlattenedPath>` containing:
    /// - The inset boundary path(s) if `params.inset` is true
    /// - The parallel fill lines clipped to the boundary
    ///
    /// Returns an empty vec if the shape is too small or fully eroded by inset.
    ///
    /// # Arguments
    /// * `params` - Hatching parameters (spacing, angle, inset, `join_lines`)
    /// * `tolerance` - Curve flattening tolerance
    ///
    /// # Errors
    /// Returns error if the path cannot be converted to a polygon
    /// (not closed, self-intersecting, etc.).
    ///
    /// # Example
    /// ```
    /// use vsvg::{Path, HatchParams, Unit};
    /// use kurbo::Circle;
    ///
    /// let circle = Path::from(Circle::new((50.0, 50.0), 25.0));
    /// let params = HatchParams::new(0.5 * Unit::Mm)
    ///     .with_angle(std::f64::consts::FRAC_PI_4);
    /// let paths = circle.hatch(&params, 0.1).unwrap();
    /// // paths contains inset boundary + diagonal fill lines
    /// ```
    pub fn hatch(
        &self,
        params: &crate::HatchParams,
        tolerance: f64,
    ) -> Result<Vec<FlattenedPath>, super::ToGeoPolygonError> {
        let polygon = self.to_geo_polygon(tolerance)?;
        let mut result = crate::hatch_polygon(&polygon, params);

        // Copy metadata to all generated paths
        for path in &mut result {
            *path.metadata_mut() = self.metadata.clone();
        }

        Ok(result)
    }
}

impl<T: IntoBezPath> From<T> for Path {
    fn from(path: T) -> Self {
        Self {
            data: path.into_bezpath(),
            ..Default::default()
        }
    }
}

impl From<FlattenedPath> for Path {
    fn from(path: FlattenedPath) -> Self {
        Self {
            data: path.data.into_bezpath(),
            metadata: path.metadata,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ToGeoPolygonError;
    use kurbo::Line;

    #[test]
    fn test_path_to_geo_polygon_circle() {
        let path = Path::from(kurbo::Circle::new((0.0, 0.0), 10.0));

        let polygon = path.to_geo_polygon(0.1).expect("should convert");
        // Circle flattened should have many points
        assert!(polygon.exterior().0.len() > 10);
        assert!(polygon.interiors().is_empty());
    }

    #[test]
    fn test_path_to_geo_polygon_square() {
        let path = Path::from_svg("M 0,0 L 10,0 L 10,10 L 0,10 Z").unwrap();

        let polygon = path.to_geo_polygon(0.1).expect("should convert");
        assert_eq!(polygon.exterior().0.len(), 5); // 4 corners + closing point
        assert!(polygon.interiors().is_empty());
    }

    #[test]
    fn test_path_to_geo_polygon_with_hole() {
        // Exterior square with interior square hole
        let path =
            Path::from_svg("M 0,0 L 10,0 L 10,10 L 0,10 Z M 3,3 L 7,3 L 7,7 L 3,7 Z").unwrap();

        let polygon = path.to_geo_polygon(0.1).expect("should convert");
        assert_eq!(polygon.interiors().len(), 1); // One hole
    }

    #[test]
    fn test_path_to_geo_polygon_open_path_error() {
        let path = Path::from_svg("M 0,0 L 10,0 L 10,10").unwrap();

        let result = path.to_geo_polygon(0.1);
        assert!(matches!(result, Err(ToGeoPolygonError::ExteriorNotClosed)));
    }

    #[test]
    fn test_path_to_geo_polygon_unclosed_hole_error() {
        // Closed exterior, unclosed interior
        let path = Path::from_svg("M 0,0 L 10,0 L 10,10 L 0,10 Z M 3,3 L 7,3 L 7,7").unwrap();

        let result = path.to_geo_polygon(0.1);
        assert!(matches!(
            result,
            Err(ToGeoPolygonError::InteriorNotClosed(0))
        ));
    }

    #[test]
    fn test_path_to_geo_polygon_empty_path() {
        let path = Path::default();
        let result = path.to_geo_polygon(0.1);
        assert!(matches!(result, Err(ToGeoPolygonError::EmptyPath)));
    }

    #[test]
    fn test_path_buffer_shrink_square() {
        let path = Path::from_svg("M 0,0 L 10,0 L 10,10 L 0,10 Z").unwrap();
        let result = path.buffer(-1.0, 0.1).expect("should buffer");

        assert_eq!(result.len(), 1); // Still one path

        // Shrunk square should have smaller bounds
        let bounds = result[0].bounds();
        // 10x10 shrunk by 1 on each side ≈ 8x8
        assert!((bounds.width() - 8.0).abs() < 0.5);
        assert!((bounds.height() - 8.0).abs() < 0.5);
    }

    #[test]
    fn test_path_buffer_expand_square() {
        let path = Path::from_svg("M 0,0 L 10,0 L 10,10 L 0,10 Z").unwrap();
        let result = path.buffer(1.0, 0.1).expect("should buffer");

        assert!(!result.is_empty());

        // Expanded square should have larger bounds
        let bounds = result[0].bounds();
        assert!(bounds.width() > 10.0);
        assert!(bounds.height() > 10.0);
    }

    #[test]
    fn test_path_buffer_completely_erodes() {
        let path = Path::from_svg("M 0,0 L 2,0 L 2,2 L 0,2 Z").unwrap();
        let result = path.buffer(-2.0, 0.1).expect("should buffer");

        // Should be empty (fully eroded)
        assert!(result.is_empty());
    }

    #[test]
    fn test_path_buffer_circle() {
        let path = Path::from(kurbo::Circle::new((0.0, 0.0), 10.0));
        let result = path.buffer(-1.0, 0.1).expect("should buffer");

        assert_eq!(result.len(), 1);
        // Shrunk circle should have smaller bounds
        let bounds = result[0].bounds();
        assert!(bounds.width() < 20.0); // Original diameter was 20
    }

    #[test]
    fn test_path_buffer_open_path_error() {
        let path = Path::from_svg("M 0,0 L 10,0").unwrap();
        let result = path.buffer(-1.0, 0.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_crop() {
        let mut path = Path::from(Line::new((0.0, 0.0), (1.0, 1.0)));
        path.crop(0.5, 0.5, 1.5, 1.5);
        let mut it = path.data.segments();
        assert_eq!(
            it.next().unwrap(),
            kurbo::PathSeg::Line(Line::new((0.5, 0.5), (1.0, 1.0)))
        );
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_path_bounds() {
        let path = Path::from(Line::new((0.0, 0.0), (1.0, 1.0)));
        assert_eq!(path.bounds(), kurbo::Rect::new(0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn test_path_start_end() {
        let path = Path::from_svg("M 0,0 L 50,110").unwrap();
        assert_eq!(path.data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(path.data.end(), Some(Point::new(50.0, 110.0)));

        let path = Path::from_svg("M 0,0 C 50,110 50,140 60,78").unwrap();
        assert_eq!(path.data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(path.data.end(), Some(Point::new(60.0, 78.0)));

        let path = Path::from_svg("M 0,0 C 50,110 50,140 60,78 Z").unwrap();
        assert_eq!(path.data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(path.data.end(), Some(Point::new(0.0, 0.0)));

        let path = Path::from_svg("M 0,0 C 50,110 50,140 60,78 M60,43 l30,50 Z").unwrap();
        assert_eq!(path.data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(path.data.end(), Some(Point::new(60.0, 43.0)));
    }

    #[test]
    fn test_path_is_point() {
        let path = Path::from_svg("M 10,0 l 0,0").unwrap();
        assert!(path.data.is_point());

        let path = Path::from_svg("M 10,0 L 10,0").unwrap();
        assert!(path.data.is_point());
    }

    #[test]
    fn test_path_split_simple() {
        // Single subpath - returns vec with one element
        let path = Path::from_svg("M 0,0 L 10,10 L 20,0").unwrap();
        let parts = path.split();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].data.elements().len(), 3); // MoveTo + 2 LineTo
    }

    #[test]
    fn test_path_split_compound() {
        // Two subpaths
        let path = Path::from_svg("M 0,0 L 10,10 M 50,50 L 60,60").unwrap();
        let parts = path.split();
        assert_eq!(parts.len(), 2);

        // First subpath: M 0,0 L 10,10
        assert_eq!(parts[0].data.elements().len(), 2);
        assert_eq!(parts[0].data.start(), Some(Point::new(0.0, 0.0)));
        assert_eq!(parts[0].data.end(), Some(Point::new(10.0, 10.0)));

        // Second subpath: M 50,50 L 60,60
        assert_eq!(parts[1].data.elements().len(), 2);
        assert_eq!(parts[1].data.start(), Some(Point::new(50.0, 50.0)));
        assert_eq!(parts[1].data.end(), Some(Point::new(60.0, 60.0)));
    }

    #[test]
    fn test_path_split_three_subpaths() {
        let path = Path::from_svg("M 0,0 L 10,0 M 20,0 L 30,0 M 40,0 L 50,0").unwrap();
        let parts = path.split();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_path_split_empty() {
        let path = Path::default();
        let parts = path.split();
        assert!(parts.is_empty());
    }

    #[test]
    fn test_path_split_with_close() {
        // Closed subpath followed by open subpath
        let path = Path::from_svg("M 0,0 L 10,0 L 10,10 Z M 50,50 L 60,60").unwrap();
        let parts = path.split();
        assert_eq!(parts.len(), 2);

        // First is closed (has ClosePath)
        assert!(
            parts[0]
                .data
                .elements()
                .iter()
                .any(|el| matches!(el, PathEl::ClosePath))
        );

        // Second is open
        assert!(
            !parts[1]
                .data
                .elements()
                .iter()
                .any(|el| matches!(el, PathEl::ClosePath))
        );
    }
}
