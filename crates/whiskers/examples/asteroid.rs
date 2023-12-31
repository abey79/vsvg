//! Asteroid design kindly contributed by @Wyth@mastodon.art for my
//! [Rusteroïds](https://github.com/abey79/rusteroids) game.

#![allow(clippy::needless_range_loop)]

use geo::{BooleanOps, BoundingRect, Contains};
use itertools::Itertools;
use rand::Rng;
use rand_distr::{Distribution, Normal};

use std::f64::consts::PI;
use whiskers::prelude::*;

#[sketch_app]
struct AsteroidSketch {
    #[param(slider, min = 0.1, max = 1.5)]
    irregularity: f64,

    #[param(slider, min = 0.0, max = 0.5)]
    spikiness: f64,

    #[param(slider, min = 3, max = 20)]
    num_vertices: usize,

    #[param(slider, min = 3, max = 10)]
    num_point: usize,

    #[param(slider, min = 1, max = 6)]
    max_iter: usize,

    #[param(slider, min = 0, max = self.max_iter-1)]
    min_iter: usize,
}

impl Default for AsteroidSketch {
    fn default() -> Self {
        Self {
            irregularity: 0.9,
            spikiness: 0.13,
            num_vertices: 18,
            num_point: 6,
            max_iter: 4,
            min_iter: 1,
        }
    }
}

impl App for AsteroidSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        vsvg::trace_function!();

        sketch
            .translate(sketch.width() / 2., sketch.height() / 2.)
            .scale(4.0 * Unit::Cm)
            .color(Color::DARK_BLUE);

        let poly = generate_polygon(
            1.0,
            self.irregularity,
            self.spikiness,
            self.num_vertices,
            &mut ctx.rng,
        );

        fn voronoi_recurse(
            sketch: &mut Sketch,
            num_point: usize,
            poly: &geo::Polygon,
            max_iter: usize,
            min_iter: usize,
            rng: &mut impl Rng,
        ) {
            let (sub_polys, segments) = voronoi(
                poly.bounding_rect(),
                &generate_points_in_poly(poly, num_point, rng),
            );

            let segments = poly.clip(&segments, false);

            sketch.add_path(segments);

            if max_iter > 0 {
                for p in &sub_polys {
                    for p in poly.intersection(p) {
                        let iter = rng.gen_range(min_iter..=max_iter);

                        if iter > 0 {
                            voronoi_recurse(
                                sketch,
                                num_point,
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

        // sanity check on iter range
        if self.min_iter >= self.max_iter {
            self.min_iter = self.max_iter.saturating_sub(1);
        }

        voronoi_recurse(
            sketch,
            self.num_point,
            &poly,
            self.max_iter,
            self.min_iter,
            &mut ctx.rng,
        );

        sketch.add_path(poly);

        Ok(())
    }
}

fn generate_polygon(
    avg_radius: f64,
    mut irregularity: f64,
    mut spikiness: f64,
    num_vertices: usize,
    rng: &mut impl Rng,
) -> geo::Polygon<f64> {
    vsvg::trace_function!();

    irregularity *= 2.0 * PI / num_vertices as f64;
    spikiness *= avg_radius;
    let normal = Normal::new(avg_radius, spikiness).unwrap();

    let angle_steps = random_angle_steps(num_vertices, irregularity, rng);

    let mut points = Vec::new();
    let mut angle = rng.gen_range(0.0..2.0 * PI);
    for i in 0..num_vertices {
        let radius = normal.sample(rng).max(0.0).min(2.0 * avg_radius);
        let point = (radius * angle.cos(), radius * angle.sin());
        points.push(point);
        angle += angle_steps[i];
    }

    geo::Polygon::new(geo::LineString::from(points), vec![])
}

fn random_angle_steps(steps: usize, irregularity: f64, rng: &mut impl Rng) -> Vec<f64> {
    vsvg::trace_function!();

    let mut angles = vec![0.0; steps];
    let lower = (2.0 * PI / (steps as f64)) - irregularity;
    let upper = (2.0 * PI / (steps as f64)) + irregularity;
    let mut cumsum = 0.0;
    for i in 0..steps {
        let angle = rng.gen_range(lower..upper);
        angles[i] = angle;
        cumsum += angle;
    }
    cumsum /= 2.0 * PI;
    for i in 0..steps {
        angles[i] /= cumsum;
    }
    angles
}

fn generate_points_in_poly(
    poly: &geo::Polygon<f64>,
    cnt: usize,
    rng: &mut impl Rng,
) -> geo::MultiPoint<f64> {
    vsvg::trace_function!();

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
    vsvg::trace_function!();

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

fn main() -> Result {
    AsteroidSketch::runner()
        .with_page_size_options(PageSize::custom(12., 12., Unit::Cm))
        .run()
}
