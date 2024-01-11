//! Example sketch showcasing the use of the `geos` crate.
//!
//! **Important**: you need `libgeos` to be installed on your system for this example to run.
//!
//! Original sketch contributed by [Daniel Simu](https://github.com/hapiel)

use geos::{CoordSeq, Geom, Geometry};
use vsvg::trace_scope;
use whiskers::prelude::*;

#[sketch_app]
struct ParticleSketch {
    pen_width: Length,
    margin: Length,
    circle_count: usize,

    #[param(slider, min = 0.1, max = 5.0, logarithmic)]
    circle_radius: Length,
}

impl Default for ParticleSketch {
    fn default() -> Self {
        Self {
            pen_width: 0.3 * Unit::Mm,
            margin: 15.0 * Unit::Mm,
            circle_count: 200,
            circle_radius: 0.5 * Unit::Cm,
        }
    }
}

impl App for ParticleSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch
            .stroke_width(self.pen_width)
            .color(Color::new(0, 0, 20, 220));

        let mut circles = vec![];

        let margin: f64 = (self.margin + self.circle_radius).into();

        {
            trace_scope!("create circles");

            for _ in 0..self.circle_count {
                let x = ctx.rng_range(margin..(sketch.width() - margin));
                let y = ctx.rng_range(margin..(sketch.height() - margin));

                let coords = CoordSeq::new_from_vec(&[&[x, y]]).expect("failed to create CoordSeq");

                let geom = Geometry::create_point(coords)
                    .unwrap()
                    .buffer(self.circle_radius.into(), 20)
                    .unwrap();

                circles.push(geom);
            }
        }

        let mut union_result: Geometry;

        {
            trace_scope!("union circles");

            let circles_multi_polygon = Geometry::create_multipolygon(circles).unwrap();
            union_result = circles_multi_polygon
                .unary_union()
                .expect("unary union failed");
            union_result.normalize().expect("normalize failed");
        }

        {
            trace_scope!("to sketch");

            let boundary = union_result.boundary().expect("boundary");

            for k in 0..boundary.get_num_geometries().expect("num geometries") {
                let geom = boundary.get_geometry_n(k).expect("geometry");
                let coords = geom.get_coord_seq().expect("coord seq");

                let pts = (0..coords.size().expect("size"))
                    .map(|i| {
                        vsvg::Point::new(coords.get_x(i).expect("x"), coords.get_y(i).expect("y"))
                    })
                    .collect::<Vec<_>>();

                sketch.add_path(&pts);
            }
        }

        Ok(())
    }
}

fn main() -> Result {
    ParticleSketch::runner()
        .with_page_size_options(PageSize::Custom(205., 130., Unit::Mm))
        .with_layout_options(LayoutOptions::Center)
        .run()
}
