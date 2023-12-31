use whiskers::prelude::*;

#[sketch_app]
struct MySketch {
    #[param(slider, min = 0.0, max = 10.0, step = 2.0)]
    rate: f64,
    num_circle: usize,
    unused_text: String,

    color: Color,

    // we can tell [`Sketch`] to ignore some fields
    #[skip]
    #[allow(dead_code)]
    irrelevant: String,
}

impl Default for MySketch {
    fn default() -> Self {
        Self {
            rate: 3.0,
            num_circle: 10,
            unused_text: "Hello".to_string(),
            color: Color::DARK_GREEN,
            irrelevant: String::new(),
        }
    }
}

impl App for MySketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.color(self.color);
        for i in 0..self.num_circle {
            sketch.circle(
                100.0,
                100.0,
                (ctx.time * self.rate).sin() * 30.0 + 40.0 + i as f64 * 3.0,
            );
        }

        Ok(())
    }
}

fn main() -> Result {
    MySketch::runner()
        .with_page_size_options(PageSize::new(200.0, 200.0))
        .run()
}
