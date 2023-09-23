use std::time::SystemTime;
use vsvg_sketch::prelude::*;

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
    fn update(
        &mut self,
        sketch: &mut Sketch,
        _rng: &mut rand_chacha::ChaCha8Rng,
    ) -> anyhow::Result<()> {
        sketch.page_size(PageSize::new(200.0, 200.0));
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs_f64();

        for i in 0..self.num_circle {
            sketch.circle(
                100.0,
                100.0,
                (now * self.rate).sin() * 30.0 + 40.0 + i as f64 * 3.0,
            );
        }

        Ok(())
    }
}

fn main() -> Result {
    run_default::<MySketch>()

    // or you can use this instead of implementing [`Default`]:
    // run(MySketch {
    //     rate: 3.0,
    //     num_circle: 10,
    //     unused_text: "Hello".to_string(),
    //     irrelevant: String::new(),
    // })
}
