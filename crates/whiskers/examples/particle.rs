use geos::{CoordSeq, Geom, Geometry};
use vsvg::trace_scope;
use whiskers::prelude::*;

#[sketch_app]
struct MySketch {
    /* add sketch parameters here */
    pen_width: f64,
    width: f64,
    height: f64,
    circle_count: usize,
    circle_radius: f64,
}

impl Default for MySketch {
    fn default() -> Self {
        Self {
            /* initialize sketch parameters to default values here */
            pen_width: 0.3,
            width: 190.0,
            height: 120.0,
            circle_count: 20,
            circle_radius: 2.,
        }
    }
}

impl App for MySketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        // draw code goes here

        sketch
            .scale(Unit::Mm)
            .stroke_width(self.pen_width)
            .color(Color::new(0, 0, 20, 220));

        let mut circles = vec![]; //MultiPolygon::new(vec![]);

        {
            trace_scope!("create circles");

            for _i in 0..self.circle_count {
                let x = _ctx.rng_range(0.0..self.width);
                let y = _ctx.rng_range(0.0..self.height);

                let coords = CoordSeq::new_from_vec(&[&[x, y]]).expect("failed to create CoordSeq");

                let geom = Geometry::create_point(coords)
                    .unwrap()
                    .buffer(self.circle_radius, 20)
                    .unwrap();

                circles.push(geom);
            }
        }

        let mut union_result: geos::Geometry; // = MultiPolygon::new(vec![circles.0[0].clone()]);

        {
            trace_scope!("union circles");

            let geom = Geometry::create_multipolygon(circles).unwrap();
            union_result = geom.unary_union().expect("unary union failed");
            union_result.normalize().expect("normalize failed");
        }

        {
            trace_scope!("to sketch");

            let ext = union_result.boundary().expect("boundary");

            for k in 0..ext.get_num_geometries().expect("num geometries") {
                let geom = ext.get_geometry_n(k).expect("geometry");
                let coords = geom.get_coord_seq().expect("coord seq");

                let pts = (0..coords.size().expect("size"))
                    .into_iter()
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
    MySketch::runner()
        .with_page_size_options(PageSize::Custom(205., 130., Unit::Mm))
        .with_layout_options(LayoutOptions::Center)
        .run()
}
