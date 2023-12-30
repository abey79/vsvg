use std::ops::Range;
use vsvg::COLORS;
use whiskers::prelude::*;

#[derive(Sketch)]
struct RngSketch {
    width: f64,
    height: f64,
}

impl Default for RngSketch {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 300.0,
        }
    }
}

impl App for RngSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.stroke_width(3.0);

        let colors = COLORS.to_vec();
        let chosen_color = ctx.rng_choice(&colors);

        let w = sketch.width();
        let h = sketch.height();
        let has_bold_stroke = ctx.rng_bool();
        let stroke_width = if has_bold_stroke { 10.0 } else { 5.0 };

        sketch
            .color(*chosen_color)
            .stroke_width(stroke_width)
            .translate(w / 2.0, h / 2.0)
            .rect(0., 0., self.width, self.height);

        let x_range = Range { start: 0.0, end: w };
        let y_range = Range { start: 0.0, end: h };

        let some_point = ctx.rng_point(x_range, y_range);

        sketch.push_matrix_reset();
        sketch.circle(some_point.x(), some_point.y(), 20.0);

        Ok(())
    }
}

fn main() -> Result {
    RngSketch::runner()
        .with_random_seed()
        .with_page_size_options(PageSize::A5H)
        .run()
}
