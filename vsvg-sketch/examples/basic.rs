use vsvg_sketch::prelude::*;

fn main() -> Result {
    Sketch::new()
        .page_size(PageSize::A5)
        .scale(Units::CM)
        .translate(7.0, 6.0)
        .circle(0.0, 0.0, 2.5)
        .translate(1.0, 4.0)
        .rotate_deg(45.0)
        .rect(0., 0., 4.0, 1.0)
        .show()?
        .save("output.svg")?;

    Ok(())
}
