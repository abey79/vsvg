use std::error::Error;
use std::f64::consts::PI;
use vsvg_sketch::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    Sketch::with_page_size(PageSize::A4)
        .scale(CM)
        .translate(7.0, 6.0)
        .circle(0.0, 0.0, 2.5)
        .translate(1.0, 4.0)
        .rotate(PI / 4.0)
        .rect(0., 0., 4.0, 1.0)
        .show()?;

    Ok(())
}
