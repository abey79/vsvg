//! This example demonstrates how to directly use [`whiskers::Sketch`] without using the [`whiskers::Runner`]
//! API.
use whiskers::prelude::*;

fn main() -> Result {
    Sketch::new()
        .page_size(PageSize::A5V)
        .scale(Unit::Cm)
        .translate(7.0, 6.0)
        .circle(0.0, 0.0, 2.5)
        .translate(1.0, 4.0)
        .rotate_deg(45.0)
        .rect(0., 0., 4.0, 1.0)
        .show()?
        .save("output.svg")?;

    Ok(())
}
