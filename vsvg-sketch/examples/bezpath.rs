//! vsvg uses [`kurbo::BezPath`] as underlying basic path structure. It's built as a list of
//! path commands including "move to", "line to", "quad bezier to", "cubic bezier to", and "close".
//! This example demonstrates how to build complex shapes by building  [`kurbo::BezPath`] instances
//! manually.

use vsvg_sketch::prelude::*;

fn main() -> Result {
    let page_size = PageSize::A5;
    let mut sketch = Sketch::new();
    sketch.page_size(page_size);

    // [`kurbo::BezPath`] a nice building API
    let mut path = kurbo::BezPath::new();
    path.move_to((0.0, 0.0));
    path.line_to((1.0, 1.0));
    path.quad_to((2.0, 0.0), (2.0, -3.0));
    path.curve_to((1.0, -2.0), (0.0, -2.0), (-1.0, -5.0));
    path.close_path();

    // it's also possible to build a path from an SVG `path` string
    let path2 = kurbo::BezPath::from_svg("M 0 0 L 1 1 Q 2 0 2 -3 C 1 -2 0 -2 -1 -5 Z")?;

    // arc commands are also supported, but converted to cubic beziers
    let path3 = kurbo::BezPath::from_svg("M 0 0 A 2 1 0 0 0 3 2 Z")?;

    sketch
        .translate(page_size.w / 2.0, 0.0)
        .scale(Units::CM)
        .translate(0.0, 7.0)
        .add_path(path)
        .translate(0.0, 6.0)
        .add_path(path2)
        .translate(0.0, 3.0)
        .add_path(path3)
        .show()?;

    Ok(())
}
