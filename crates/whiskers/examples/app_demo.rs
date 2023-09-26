use whiskers::prelude::*;

#[derive(Sketch)]
struct MySketch {
    #[param(slider, min = 0.0, max = 10.0, step = 2.0)]
    rate: f64,
    num_circle: usize,
    unused_text: String,

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
            irrelevant: String::new(),
        }
    }
}

impl App for MySketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
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
    Runner::new(MySketch::default())
        .with_page_size(PageSize::new(200.0, 200.0))
        .run()
}
