use arrayvec::ArrayVec;
use kurbo::CubicBez;
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
        p0: kurbo::Point::new(cbs.from.x, cbs.from.y),
        p1: kurbo::Point::new(cbs.ctrl1.x, cbs.ctrl1.y),
        p2: kurbo::Point::new(cbs.ctrl2.x, cbs.ctrl2.y),
        p3: kurbo::Point::new(cbs.to.x, cbs.to.y),
    }
}

fn crop_x(bez: CubicBez, x: f64, keep_smaller: bool) -> ArrayVec<CubicBez, 3> {
    let cbs = k2l(bez);
    let mut intsct = cbs.solve_t_for_x(x);

    // Strategy:
    // - Sort intersections by increasing t.
    // - Preprocess the interesction list by removing out-of-bound intersections. This includes
    //   slightly in-bound intersections, effectively snapping extremities to the crop line.
    // - Filter out interesections at extremities.
    // - Keep only those ranges between intersections that are in the correct half-plane.
    // - Merge contiguous ranges (happens when the curve is tangent to the crop line).

    intsct.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut prev_t = 0.;
    let keep_range: ArrayVec<Range<_>, 4> = intsct
        .as_slice()
        .iter()
        .copied()
        .filter(|&t| t > 10. * f64::EPSILON && t < 1.0 - 10. * f64::EPSILON)
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use kurbo::{ParamCurve, Point};

    fn assert_approx_point(a: Point, b: Point) {
        assert_abs_diff_eq!(a.x, b.x, epsilon = 1e-10);
        assert_abs_diff_eq!(a.y, b.y, epsilon = 1e-10);
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
    fn test_crop_x_symmetrical_s_shape() {
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
        assert_bezvec_eq(
            crop_x(bez, 50., true).as_slice(),
            &[bez.subsegment(0.0..1.)],
        );

        assert_bezvec_eq(crop_x(bez, 50., false).as_slice(), &[]);

        // symmetrical cut
        assert_bezvec_eq(
            crop_x(bez, 0., true).as_slice(),
            &[bez.subsegment(0.0..0.5)],
        );

        assert_bezvec_eq(
            crop_x(bez, 0., false).as_slice(),
            &[bez.subsegment(0.5..1.)],
        );

        // tengant cuts
        let (x_min, x_max) = cbs.bounding_range_x();
        assert_bezvec_eq(
            crop_x(bez, x_max, true).as_slice(),
            &[bez.subsegment(0.0..1.)],
        );

        assert_bezvec_eq(crop_x(bez, x_max, false).as_slice(), &[]);
        assert_bezvec_eq(
            crop_x(bez, x_min, false).as_slice(),
            &[bez.subsegment(0.0..1.)],
        );
        assert_bezvec_eq(crop_x(bez, x_min, true).as_slice(), &[]);
    }
}
