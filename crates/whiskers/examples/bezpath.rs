//! vsvg uses [`kurbo::BezPath`] as underlying basic path structure. It's built as a list of
//! path commands including "move to", "line to", "quad bezier to", "cubic bezier to", and "close".
//! This example demonstrates how to build complex shapes by building  [`kurbo::BezPath`] instances
//! manually.

use whiskers::prelude::*;

#[derive(Sketch)]
struct BezpathSketch {
    // path 1
    move_to: Point,
    line_to: Point,
    quad_to_0: Point,
    quad_to_1: Point,
    curve_to_0: Point,
    curve_to_1: Point,
    curve_to_2: Point,
    close: bool,

    // path 2
    path2_svg: String,

    // path 3
    path3_svg: String,
}

impl Default for BezpathSketch {
    fn default() -> Self {
        Self {
            move_to: Point::new(0.0, 0.0),
            line_to: Point::new(1.0, 1.0),
            quad_to_0: Point::new(2.0, 0.0),
            quad_to_1: Point::new(2.0, -3.0),
            curve_to_0: Point::new(1.0, -2.0),
            curve_to_1: Point::new(0.0, -2.0),
            curve_to_2: Point::new(-1.0, -5.0),
            close: true,

            // it's also possible to build a path from an SVG `path` string
            path2_svg: "M 0 0 L 1 1 Q 2 0 2 -3 C 1 -2 0 -2 -1 -5 Z".to_string(),

            // arc commands are also supported, but converted to cubic Béziers
            path3_svg: "M 0 0 A 2 1 0 0 0 3 2 Z".to_string(),
        }
    }
}

impl App for BezpathSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        let mut path = kurbo::BezPath::new();
        path.move_to(self.move_to);
        path.line_to(self.line_to);
        path.quad_to(self.quad_to_0, self.quad_to_1);
        path.curve_to(self.curve_to_0, self.curve_to_1, self.curve_to_2);
        if self.close {
            path.close_path();
        }

        let path2 = kurbo::BezPath::from_svg(self.path2_svg.as_str()).ok();
        let path3 = kurbo::BezPath::from_svg(self.path3_svg.as_str()).ok();

        fn paint_cross(sketch: &mut Sketch) {
            sketch
                .color(Color::RED)
                .stroke_width(0.5)
                .line(-0.1, 0.0, 0.1, 0.0)
                .line(0.0, 0.1, 0.0, -0.1)
                .color(Color::BLACK)
                .stroke_width(1.0);
        }

        sketch.scale_unit(Unit::CM).translate(6.0, 7.0);

        paint_cross(sketch);
        sketch.add_path(path).translate(0.0, 6.0);

        paint_cross(sketch);
        if let Some(path2) = path2 {
            sketch.add_path(path2);
        }

        sketch.translate(0.0, 3.0);

        paint_cross(sketch);
        if let Some(path3) = path3 {
            sketch.add_path(path3);
        }

        Ok(())
    }
}

fn main() -> Result {
    Runner::new(BezpathSketch::default())
        .with_locked_page_size(PageSize::A5V)
        .with_time_enabled(false)
        .with_seed_enabled(false)
        .run()
}
