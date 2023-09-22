use std::time::SystemTime;
use vsvg_sketch::prelude::*;

fn main() -> Result {
    SketchRunner::new(MySketch).run()
}

struct MySketch;

impl SketchApp for MySketch {
    fn update(&mut self, sketch: &mut Sketch) -> anyhow::Result<()> {
        sketch.page_size(PageSize::new(200.0, 200.0));
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs_f64();
        sketch.circle(100.0, 100.0, (now * 4.0).sin() * 30.0 + 50.0);

        Ok(())
    }
}
