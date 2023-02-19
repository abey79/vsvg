use arrayvec::ArrayVec;
use kurbo::{CubicBez, Line, Point};
use std::ops::Range;

//TODO: remove dependency on lyon_geom by reimplementing the x-axis intersection code.

fn k2l(bez: CubicBez) -> lyon_geom::CubicBezierSegment<f64> {
    lyon_geom::CubicBezierSegment {
        from: lyon_geom::point(bez.p0.x, bez.p0.y),
        ctrl1: lyon_geom::point(bez.p1.x, bez.p1.y),
        ctrl2: lyon_geom::point(bez.p2.x, bez.p2.y),
        to: lyon_geom::point(bez.p3.x, bez.p3.y),
    }
}

fn l2k(cbs: lyon_geom::CubicBezierSegment<f64>) -> CubicBez {
    CubicBez {
        p0: Point::new(cbs.from.x, cbs.from.y),
        p1: Point::new(cbs.ctrl1.x, cbs.ctrl1.y),
        p2: Point::new(cbs.ctrl2.x, cbs.ctrl2.y),
        p3: Point::new(cbs.to.x, cbs.to.y),
    }
}

const EPSILON: f64 = 10. * f64::EPSILON;

/// Implement cropability for a geometric type.
pub trait Crop<const N: usize>
where
    Self: Sized,
{
    /// Crop the geometry to a given rectangle.
    fn crop(self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Vec<Self> {
        self.crop_x(x_min, false)
            .into_iter()
            .flat_map(|bez| bez.crop_x(x_max, true))
            .flat_map(|bez| bez.crop_y(y_min, false))
            .flat_map(|bez| bez.crop_y(y_max, true))
            .collect()
    }

    /// Transpose the geometry axes. This is used to provide a default implementation of `crop_y`
    /// using `crop_x`.
    fn transpose_axes(self) -> Self;

    /// Crop the geometry to a given vertically-split, half-plane defined with a X coordinate.
    fn crop_x(self, x: f64, keep_smaller: bool) -> ArrayVec<Self, N>;

    /// Crop the geometry to a given horizontally-split, half-plane defined with a X coordinate.
    fn crop_y(self, y: f64, keep_smaller: bool) -> ArrayVec<Self, N> {
        self.transpose_axes()
            .crop_x(y, keep_smaller)
            .into_iter()
            .map(Self::transpose_axes)
            .collect()
    }
}

impl Crop<2> for Line {
    fn transpose_axes(self) -> Self {
        Line::new(
            Point::new(self.p0.y, self.p0.x),
            Point::new(self.p1.y, self.p1.x),
        )
    }

    fn crop_x(self, x: f64, keep_smaller: bool) -> ArrayVec<Self, 2> {
        let line = lyon_geom::LineSegment {
            from: lyon_geom::point(self.p0.x, self.p0.y),
            to: lyon_geom::point(self.p1.x, self.p1.y),
        };

        let mut out = ArrayVec::new();

        if let Some(mut t) = line.line_intersection_t(&lyon_geom::Line {
            point: lyon_geom::point(x, 0.),
            vector: lyon_geom::vector(0., 1.),
        }) {
            if t < EPSILON {
                t = 0.;
            } else if t > 1. - EPSILON {
                t = 1.;
            }

            let first_in = (line.from.x < x) == keep_smaller;
            let last_in = (line.to.x < x) == keep_smaller;

            if t == 0. {
                if last_in {
                    out.push(self);
                }
            } else if t == 1. {
                if first_in {
                    out.push(self);
                }
            } else {
                let p = line.from.lerp(line.to, t);
                if first_in {
                    out.push(Line {
                        p0: Point {
                            x: self.p0.x,
                            y: self.p0.y,
                        },
                        p1: Point { x: p.x, y: p.y },
                    });
                } else {
                    out.push(Line {
                        p0: Point { x: p.x, y: p.y },
                        p1: Point {
                            x: self.p1.x,
                            y: self.p1.y,
                        },
                    });
                }
            }
        } else {
            // No intersection
            if (line.from.x < x) == keep_smaller || line.from.x == x {
                out.push(self);
            }
        }

        out
    }
}

impl Crop<3> for CubicBez {
    fn transpose_axes(self) -> Self {
        CubicBez {
            p0: Point::new(self.p0.y, self.p0.x),
            p1: Point::new(self.p1.y, self.p1.x),
            p2: Point::new(self.p2.y, self.p2.x),
            p3: Point::new(self.p3.y, self.p3.x),
        }
    }

    fn crop_x(self, x: f64, keep_smaller: bool) -> ArrayVec<Self, 3> {
        let cbs = k2l(self);
        let mut intsct = cbs.solve_t_for_x(x);

        // Strategy:
        // - Sort intersections by increasing t.
        // - Preprocess the intersection list by removing out-of-bound intersections. This includes
        //   slightly in-bound intersections, effectively snapping extremities to the crop line.
        // - Filter out intersections at extremities.
        // - Keep only those ranges between intersections that are in the correct half-plane.
        // - Merge contiguous ranges (happens when the curve is tangent to the crop line).

        intsct.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut prev_t = 0.;
        let keep_range: ArrayVec<Range<_>, 4> = intsct
            .as_slice()
            .iter()
            .copied()
            .filter(|&t| t > EPSILON && t < 1.0 - EPSILON)
            .chain([1.0])
            .filter_map(|t| {
                let res = if (cbs.sample((prev_t + t) * 0.5).x < x) == keep_smaller {
                    Some(prev_t..t)
                } else {
                    None
                };
                prev_t = t;
                res
            })
            .collect();

        // merge contiguous ranges
        let mut merged = ArrayVec::<Range<_>, 4>::new();
        for r in keep_range {
            if let Some(last) = merged.last_mut() {
                if last.end == r.start {
                    last.end = r.end;
                    continue;
                }
            }
            merged.push(r);
        }

        merged
            .into_iter()
            .map(|r| l2k(cbs.split_range(r)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use kurbo::{ParamCurve, Point};

    fn assert_approx_point(a: Point, b: Point) {
        assert_abs_diff_eq!(a.x, b.x, epsilon = 1e-10);
        assert_abs_diff_eq!(a.y, b.y, epsilon = 1e-10);
    }

    fn assert_line_eq(actual: &[Line], expected: &[Line]) {
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected.iter()) {
            assert_approx_point(a.p0, b.p0);
            assert_approx_point(a.p1, b.p1);
        }
    }

    fn assert_bezvec_eq(actual: &[CubicBez], expected: &[CubicBez]) {
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected.iter()) {
            assert_approx_point(a.p0, b.p0);
            assert_approx_point(a.p1, b.p1);
            assert_approx_point(a.p2, b.p2);
            assert_approx_point(a.p3, b.p3);
        }
    }

    #[test]
    fn test_crop_line() {
        let line = Line::new(Point::new(-2., -2.), Point::new(1., 1.));

        assert_line_eq(
            line.crop_x(0., true).as_slice(),
            &[Line::new(Point::new(-2., -2.), Point::new(0., 0.))],
        );
        assert_line_eq(
            line.crop_x(0., false).as_slice(),
            &[Line::new(Point::new(0., 0.), Point::new(1., 1.))],
        );

        assert_line_eq(line.crop_x(-2., true).as_slice(), &[]);
        assert_line_eq(line.crop_x(-2., false).as_slice(), &[line]);

        assert_line_eq(line.crop_x(1., false).as_slice(), &[]);
        assert_line_eq(line.crop_x(1., true).as_slice(), &[line]);
    }

    #[test]
    fn test_crop_line_colinear() {
        let line = Line::new(Point::new(0., 0.), Point::new(0., 5.));

        assert_line_eq(line.crop_x(0., true).as_slice(), &[line]);
        assert_line_eq(line.crop_x(0., false).as_slice(), &[line]);

        assert_line_eq(line.crop_x(5., true).as_slice(), &[line]);
        assert_line_eq(line.crop_x(5., false).as_slice(), &[]);

        assert_line_eq(line.crop_x(-5., false).as_slice(), &[line]);
        assert_line_eq(line.crop_x(-5., true).as_slice(), &[]);
    }

    #[test]
    fn test_crop_x_symmetrical_s_bezier() {
        // S shaped bezier along x, with symmetrical control points
        // start, ends and mid-point at x = 0
        let bez = CubicBez::new(
            Point::new(0.0, 0.0),
            Point::new(-5.0, 1.0),
            Point::new(5.0, 2.0),
            Point::new(0.0, 3.0),
        );
        let cbs = k2l(bez);

        // far off cut
        assert_bezvec_eq(bez.crop_x(50., true).as_slice(), &[bez.subsegment(0.0..1.)]);
        assert_bezvec_eq(bez.crop_x(50., false).as_slice(), &[]);

        // symmetrical cut
        assert_bezvec_eq(bez.crop_x(0., true).as_slice(), &[bez.subsegment(0.0..0.5)]);
        assert_bezvec_eq(bez.crop_x(0., false).as_slice(), &[bez.subsegment(0.5..1.)]);

        // tangent cuts
        let (x_min, x_max) = cbs.bounding_range_x();
        assert_bezvec_eq(
            bez.crop_x(x_max, true).as_slice(),
            &[bez.subsegment(0.0..1.)],
        );

        assert_bezvec_eq(bez.crop_x(x_max, false).as_slice(), &[]);
        assert_bezvec_eq(
            bez.crop_x(x_min, false).as_slice(),
            &[bez.subsegment(0.0..1.)],
        );
        assert_bezvec_eq(bez.crop_x(x_min, true).as_slice(), &[]);
    }
}
