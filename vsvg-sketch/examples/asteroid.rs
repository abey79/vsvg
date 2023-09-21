//! Asteroid design kindly contributed by @Wyth@mastodon.art for my
//! [Rusteroïds](https://github.com/abey79/rusteroids) game.

use geo::{BooleanOps, BoundingRect, Contains};
use itertools::Itertools;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};

use std::f64::consts::PI;
use vsvg::Color;
use vsvg_sketch::prelude::*;

fn main() -> Result {
    let mut rng = ChaCha8Rng::seed_from_u64(4);

    let mut sketch = Sketch::new();

    let page_size = PageSize::A5;
    sketch
        .page_size(page_size)
        .translate(page_size.w / 2.0, page_size.h / 2.0)
        .scale(4.0 * Units::CM);

    let poly = generate_polygon(1.0, 0.9, 0.13, 18, &mut rng);

    fn voronoi_recurse(
        sketch: &mut Sketch,
        poly: &geo::Polygon,
        max_iter: usize,
        min_iter: usize,
        rng: &mut impl Rng,
    ) {
        let (sub_polys, segments) =
            voronoi(poly.bounding_rect(), &generate_points_in_poly(poly, 3, rng));

        let segments = poly.clip(&segments, false);

        sketch
            .stroke_width(3.0)
            .color(Color::DARK_GREEN.with_opacity(0.3))
            .add_path(segments);

        if max_iter > 0 {
            for p in &sub_polys {
                for p in poly.intersection(p) {
                    let iter = rng.gen_range(min_iter..=max_iter);

                    if iter > 0 {
                        voronoi_recurse(
                            sketch,
                            &p,
                            max_iter.saturating_sub(1),
                            min_iter.saturating_sub(1),
                            rng,
                        );
                    }
                }
            }
        }
    }

    voronoi_recurse(&mut sketch, &poly, 4, 1, &mut rng);

    sketch
        .color(Color::DARK_BLUE.with_opacity(0.3))
        .add_path(poly)
        .show()?;

    Ok(())
}

fn generate_polygon(
    avg_radius: f64,
    mut irregularity: f64,
    mut spikiness: f64,
    num_vertices: usize,
    rng: &mut impl Rng,
) -> geo::Polygon<f64> {
    irregularity *= 2.0 * PI / num_vertices as f64;
    spikiness *= avg_radius;
    let normal = Normal::new(avg_radius, spikiness).unwrap();

    let mut points = Vec::new();
    let mut angle = rng.gen_range(0.0..2.0 * PI);
    for _ in 0..num_vertices {
        let radius = normal.sample(rng).max(0.0).min(2.0 * avg_radius);
        let point = (radius * angle.cos(), radius * angle.sin());
        points.push(point);
        angle += irregularity;
    }

    geo::Polygon::new(geo::LineString::from(points), vec![])
}

fn generate_points_in_poly(
    poly: &geo::Polygon<f64>,
    cnt: usize,
    rng: &mut impl Rng,
) -> geo::MultiPoint<f64> {
    let Some(bbox) = poly.bounding_rect() else {
        return geo::MultiPoint::<f64>::new(vec![]);
    };

    let mut points = geo::MultiPoint::<f64>::new(Vec::with_capacity(cnt));
    while points.0.len() < cnt {
        let pt = geo::Coord::<f64> {
            x: rng.gen_range(bbox.min().x..bbox.max().x),
            y: rng.gen_range(bbox.min().y..bbox.max().y),
        }
        .into();
        if poly.contains(&pt) {
            points.0.push(pt);
        }
    }

    points
}

fn voronoi(
    bbox: Option<geo::Rect<f64>>,
    points: &geo::MultiPoint<f64>,
) -> (geo::MultiPolygon<f64>, geo::MultiLineString<f64>) {
    let bbox = bbox.map(|r| {
        voronoice::BoundingBox::new(
            voronoice::Point {
                x: r.center().x,
                y: r.center().y,
            },
            1.5 * r.width(), // increase slightly bbox to avoid nasty intersections
            1.5 * r.height(),
        )
    });

    let mut my_voronoi = voronoice::VoronoiBuilder::default().set_sites(
        points
            .into_iter()
            .map(|pt| voronoice::Point {
                x: pt.x(),
                y: pt.y(),
            })
            .collect(),
    );

    if let Some(bbox) = bbox {
        my_voronoi = my_voronoi.set_bounding_box(bbox);
    }

    let v = my_voronoi.build().unwrap();

    fn point_to_coord(p: &voronoice::Point) -> geo::Coord<f64> {
        geo::Coord::<f64> { x: p.x, y: p.y }
    }

    let segments = geo::MultiLineString(
        v.cells()
            .iter()
            .flat_map(|cell| {
                cell.windows(2)
                    .map(|pts| (pts[0], pts[1]))
                    .chain([(cell[cell.len() - 1], cell[0])])
                    .map(|(a, b)| if a > b { (b, a) } else { (a, b) })
            })
            .unique()
            .map(|(a, b)| {
                geo::LineString(vec![
                    point_to_coord(&v.vertices()[a]),
                    point_to_coord(&v.vertices()[b]),
                ])
            })
            .collect(),
    );

    let polys: geo::MultiPolygon<f64> = geo::MultiPolygon::new(
        v.cells()
            .iter()
            .map(|cell| {
                geo::Polygon::new(
                    geo::LineString(
                        cell.iter()
                            .map(|p| point_to_coord(&v.vertices()[*p]))
                            .collect(),
                    ),
                    vec![],
                )
            })
            .collect(),
    );

    (polys, segments)
}
