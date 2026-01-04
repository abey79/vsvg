//! Hatching algorithm for filling closed shapes with parallel lines.
//!
//! Plotters can only draw lines. To simulate filled shapes, we use hatching - parallel lines
//! clipped to the shape boundary.
//!
//! # Example
//! ```
//! use vsvg::{Path, HatchParams, Unit};
//! use kurbo::Circle;
//!
//! let circle = Path::from(Circle::new((50.0, 50.0), 25.0));
//! let params = HatchParams::new(0.5 * Unit::Mm)
//!     .with_angle(std::f64::consts::FRAC_PI_4);
//! let paths = circle.hatch(&params, 0.1).unwrap();
//! // paths contains inset boundary + diagonal fill lines
//! ```

use geo::Buffer;
use geo::algorithm::bool_ops::BooleanOps;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::algorithm::centroid::Centroid;
use geo::algorithm::rotate::Rotate;

use crate::Length;
use crate::path::{FlattenedPath, Point, Polyline};

/// Parameters for hatching a closed shape.
#[derive(Debug, Clone, Copy)]
pub struct HatchParams {
    /// Spacing between hatch lines in pixels (typically `pen_width`).
    /// Use `new()` with `impl Into<Length>` to set this with units.
    pub spacing: f64,

    /// Angle of hatch lines in radians (0 = horizontal, PI/2 = vertical).
    pub angle: f64,

    /// If true, inset the boundary by spacing/2 and include the inset boundary
    /// as a stroke in the result. Default: true.
    ///
    /// When enabled, the result includes:
    /// 1. The inset boundary path(s)
    /// 2. The hatch lines clipped to the inset boundary
    pub inset: bool,

    /// If true, run line joining on hatch lines with tolerance = 5 * spacing.
    /// This merges adjacent hatch lines for efficient plotting.
    /// Default: true.
    ///
    /// When disabled, consider using zigzag line generation for efficiency.
    pub join_lines: bool,
}

impl HatchParams {
    /// Create new hatch parameters with the given spacing.
    ///
    /// Accepts any type that converts to [`Length`], including:
    /// - `f64` (raw pixels)
    /// - `1.0 * Unit::Mm` (millimeters)
    /// - `Length::new(1.0, Unit::Cm)` (centimeters)
    ///
    /// Defaults: angle = 0 (horizontal), inset = true, `join_lines` = true.
    ///
    /// # Example
    /// ```
    /// use vsvg::{HatchParams, Unit};
    ///
    /// // Using millimeters (typical pen width)
    /// let params = HatchParams::new(0.5 * Unit::Mm);
    ///
    /// // Using raw pixels
    /// let params = HatchParams::new(2.0);
    /// ```
    #[must_use]
    pub fn new(spacing: impl Into<Length>) -> Self {
        let spacing_length: Length = spacing.into();
        Self {
            spacing: spacing_length.into(), // Convert Length to f64 (pixels)
            angle: 0.0,
            inset: true,
            join_lines: true,
        }
    }

    /// Set the hatch angle in radians.
    #[must_use]
    pub fn with_angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    /// Set whether to inset the boundary. When true (default), the result includes
    /// the inset boundary stroke plus hatch lines.
    #[must_use]
    pub fn with_inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// Set whether to join adjacent hatch lines (tolerance = 5 * spacing).
    /// When disabled, hatch lines are returned as-is.
    #[must_use]
    pub fn with_join_lines(mut self, join: bool) -> Self {
        self.join_lines = join;
        self
    }
}

/// Convert a polygon to boundary paths (exterior + holes).
fn polygon_to_boundary_paths(polygon: &geo::Polygon<f64>) -> Vec<FlattenedPath> {
    let mut paths = Vec::new();

    // Exterior ring
    let exterior_points: Vec<Point> = polygon
        .exterior()
        .0
        .iter()
        .map(|c| Point::new(c.x, c.y))
        .collect();
    if exterior_points.len() >= 3 {
        paths.push(FlattenedPath::from(Polyline::new(exterior_points)));
    }

    // Interior rings (holes)
    for interior in polygon.interiors() {
        let hole_points: Vec<Point> = interior.0.iter().map(|c| Point::new(c.x, c.y)).collect();
        if hole_points.len() >= 3 {
            paths.push(FlattenedPath::from(Polyline::new(hole_points)));
        }
    }

    paths
}

/// Generate hatching for a `geo::Polygon`.
///
/// This is the core algorithm. Use [`crate::Path::hatch`] or [`Polyline::hatch`] for
/// convenience methods that handle conversion.
///
/// # Arguments
/// * `polygon` - A valid, closed polygon (may have holes)
/// * `params` - Hatching parameters
///
/// # Returns
/// A `Vec<FlattenedPath>` containing boundary paths (if inset enabled) and hatch lines.
/// Returns empty vec if shape is fully eroded.
#[must_use]
pub fn hatch_polygon(polygon: &geo::Polygon<f64>, params: &HatchParams) -> Vec<FlattenedPath> {
    // Early return for invalid spacing
    if params.spacing <= 0.0 {
        return vec![];
    }

    let mut result: Vec<FlattenedPath> = Vec::new();

    // Step 3: Inset and boundary extraction
    let work_polygon = if params.inset {
        let inset_distance = -params.spacing / 2.0;
        let multi = polygon.buffer(inset_distance);

        // Buffer can return multiple polygons (if shape splits) or empty (eroded)
        if multi.0.is_empty() {
            return vec![]; // Fully eroded
        }

        // Convert all inset polygons to boundary paths, add to result
        for poly in &multi.0 {
            result.extend(polygon_to_boundary_paths(poly));
        }

        // Use the largest polygon for hatching
        // Safety: we checked is_empty() above, so max_by always returns Some
        let Some(largest) = multi.0.into_iter().max_by(|a, b| {
            use geo::algorithm::area::Area;
            a.unsigned_area()
                .partial_cmp(&b.unsigned_area())
                .unwrap_or(std::cmp::Ordering::Equal)
        }) else {
            return result; // Should never happen, but handle gracefully
        };
        largest
    } else {
        polygon.clone()
    };

    // Get centroid for rotation
    let centroid = work_polygon.centroid().unwrap_or(geo::Point::new(0.0, 0.0));

    // Step 5: Rotate polygon so hatch lines become horizontal
    let rotated_poly = work_polygon.rotate_around_point(-params.angle.to_degrees(), centroid);

    // Get bounds of rotated polygon
    let Some(bounds) = rotated_poly.bounding_rect() else {
        return result; // Degenerate polygon
    };
    let (x_min, x_max) = (bounds.min().x - 1.0, bounds.max().x + 1.0);
    let (y_min, y_max) = (bounds.min().y, bounds.max().y);

    // Step 6: Generate horizontal scan lines as MultiLineString
    let mut lines: Vec<geo::LineString<f64>> = Vec::new();
    let mut y = y_min + params.spacing / 2.0; // Start half-spacing from edge

    while y < y_max {
        let line =
            geo::LineString::new(vec![geo::Coord { x: x_min, y }, geo::Coord { x: x_max, y }]);
        lines.push(line);
        y += params.spacing;
    }

    if lines.is_empty() {
        return result; // Shape too small for any hatch lines
    }

    let scan_lines = geo::MultiLineString::new(lines);

    // Step 7: Clip scan lines against the rotated polygon
    // invert=false means keep the parts INSIDE the polygon
    let clipped: geo::MultiLineString<f64> = rotated_poly.clip(&scan_lines, false);

    // Step 8: Rotate clipped lines back to original orientation
    let result_lines: geo::MultiLineString<f64> =
        clipped.rotate_around_point(params.angle.to_degrees(), centroid);

    // Step 9: Convert clipped lines to FlattenedPath, add to result
    let hatch_lines: Vec<FlattenedPath> = result_lines
        .0
        .into_iter()
        .filter(|ls| ls.0.len() >= 2)
        .map(|ls| {
            let points: Vec<Point> = ls.0.iter().map(|c| Point::new(c.x, c.y)).collect();
            FlattenedPath::from(Polyline::new(points))
        })
        .collect();

    result.extend(hatch_lines);

    // Step 10: Optional line joining for efficiency
    if params.join_lines && result.len() > 1 {
        let join_tolerance = params.spacing * 5.0;
        crate::optimization::join_paths(&mut result, join_tolerance, true);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_square_polygon(size: f64) -> geo::Polygon<f64> {
        geo::Polygon::new(
            geo::LineString::new(vec![
                geo::Coord { x: 0.0, y: 0.0 },
                geo::Coord { x: size, y: 0.0 },
                geo::Coord { x: size, y: size },
                geo::Coord { x: 0.0, y: size },
                geo::Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        )
    }

    fn make_circle_linestring(cx: f64, cy: f64, r: f64, n: usize) -> geo::LineString<f64> {
        let coords: Vec<_> = (0..=n)
            .map(|i| {
                let theta = 2.0 * std::f64::consts::PI * (i % n) as f64 / n as f64;
                geo::Coord {
                    x: cx + r * theta.cos(),
                    y: cy + r * theta.sin(),
                }
            })
            .collect();
        geo::LineString::new(coords)
    }

    #[test]
    fn hatch_simple_square() {
        let polygon = make_square_polygon(10.0);
        let params = HatchParams::new(2.0)
            .with_inset(false)
            .with_join_lines(false);
        let paths = hatch_polygon(&polygon, &params);

        // ~5 horizontal lines expected (no boundary since inset=false, no joining)
        assert!(!paths.is_empty());
        assert!(paths.len() >= 3);
    }

    #[test]
    fn hatch_with_inset_includes_boundary() {
        let polygon = make_square_polygon(10.0);
        let params = HatchParams::new(2.0); // inset=true by default

        let paths = hatch_polygon(&polygon, &params);

        // Should have boundary + hatch lines
        assert!(!paths.is_empty());

        // More paths than without inset (boundary adds paths)
        let params_no_inset = HatchParams::new(2.0).with_inset(false);
        let paths_no_inset = hatch_polygon(&polygon, &params_no_inset);
        assert!(paths.len() >= paths_no_inset.len());
    }

    #[test]
    fn hatch_square_with_angle() {
        let polygon = make_square_polygon(10.0);
        let params = HatchParams::new(1.0)
            .with_angle(std::f64::consts::FRAC_PI_4)
            .with_inset(false);

        let paths = hatch_polygon(&polygon, &params);
        assert!(!paths.is_empty());
    }

    #[test]
    fn hatch_with_hole() {
        // Square with square hole
        let exterior = geo::LineString::new(vec![
            geo::Coord { x: 0.0, y: 0.0 },
            geo::Coord { x: 10.0, y: 0.0 },
            geo::Coord { x: 10.0, y: 10.0 },
            geo::Coord { x: 0.0, y: 10.0 },
            geo::Coord { x: 0.0, y: 0.0 },
        ]);
        let hole = geo::LineString::new(vec![
            geo::Coord { x: 3.0, y: 3.0 },
            geo::Coord { x: 7.0, y: 3.0 },
            geo::Coord { x: 7.0, y: 7.0 },
            geo::Coord { x: 3.0, y: 7.0 },
            geo::Coord { x: 3.0, y: 3.0 },
        ]);
        let polygon = geo::Polygon::new(exterior, vec![hole]);

        let params = HatchParams::new(1.0).with_inset(false);
        let paths = hatch_polygon(&polygon, &params);

        // Lines through center should be split by hole
        assert!(!paths.is_empty());
    }

    #[test]
    fn hatch_fully_eroded() {
        let polygon = make_square_polygon(2.0); // Small square
        let params = HatchParams::new(4.0); // Large spacing, insets by 2.0

        let paths = hatch_polygon(&polygon, &params);
        assert!(paths.is_empty());
    }

    #[test]
    fn hatch_with_line_joining() {
        let polygon = make_square_polygon(10.0);
        let params = HatchParams::new(1.0)
            .with_inset(false)
            .with_join_lines(true);

        let paths = hatch_polygon(&polygon, &params);

        // With joining, should have fewer paths than without
        let params_no_join = HatchParams::new(1.0)
            .with_inset(false)
            .with_join_lines(false);
        let paths_no_join = hatch_polygon(&polygon, &params_no_join);

        assert!(paths.len() <= paths_no_join.len());
    }

    #[test]
    fn hatch_circle() {
        let circle = geo::Polygon::new(make_circle_linestring(50.0, 50.0, 25.0, 64), vec![]);
        let params = HatchParams::new(2.0);
        let paths = hatch_polygon(&circle, &params);

        assert!(!paths.is_empty());
    }

    #[test]
    fn hatch_zero_spacing_returns_empty() {
        let polygon = make_square_polygon(10.0);
        let params = HatchParams::new(0.0);
        let paths = hatch_polygon(&polygon, &params);
        assert!(paths.is_empty());
    }

    #[test]
    fn hatch_negative_spacing_returns_empty() {
        let polygon = make_square_polygon(10.0);
        let params = HatchParams::new(-1.0);
        let paths = hatch_polygon(&polygon, &params);
        assert!(paths.is_empty());
    }
}
