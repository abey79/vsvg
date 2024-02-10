use std::ops::Range;
use vsvg::COLORS;
use whiskers::prelude::*;

#[sketch_app]
struct RngSketch {
    width: f64,
    height: f64,
    choose_color_with_weight: bool,
}

impl Default for RngSketch {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 300.0,
            choose_color_with_weight: false,
        }
    }
}

impl App for RngSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.stroke_width(3.0);

        let w = sketch.width();
        let h = sketch.height();

        let weighted_colors = vec![
            (20.0, Color::RED),
            (10.0, Color::YELLOW),
            (5.0, Color::LIGHT_YELLOW),
            (30.0, Color::GREEN),
            (5.0, Color::LIGHT_GREEN),
            (20.0, Color::BLUE),
            (10.0, Color::LIGHT_BLUE),
        ];
        let all_colors = COLORS.to_vec();
        let chosen_color = if self.choose_color_with_weight {
            ctx.rng_weighted_choice(&weighted_colors)
        } else {
            ctx.rng_choice(&all_colors)
        };

        let has_bold_stroke = ctx.rng_bool();
        let stroke_width = if has_bold_stroke { 10.0 } else { 5.0 };

        let x_range = Range { start: 0.0, end: w };
        let y_range = Range { start: 0.0, end: h };

        let some_point = ctx.rng_point(x_range, y_range);

        sketch
            .color(*chosen_color)
            .stroke_width(stroke_width)
            .translate(w / 2.0, h / 2.0)
            .rect(0., 0., self.width, self.height);

        sketch.push_matrix_reset();
        sketch.circle(some_point.x(), some_point.y(), 20.0);

        Ok(())
    }
}

fn main() -> Result {
    RngSketch::runner()
        .with_debug_options(DebugOptions::default().label("Features"))
        .with_random_seed()
        .with_page_size_options(PageSize::A5H)
        .run()
}
