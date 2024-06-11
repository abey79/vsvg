//! This example shows how to use [`kurbo::dash`] to create dashed lines in a sketch.

use vsvg::PathDataTrait;
use whiskers::prelude::*;

#[sketch_app]
struct DashedLinesSketch {
    #[param(slider, min = 0.0, max = 10.0)]
    offset: Length,

    //TODO: this really needs the vector feature for arbitrary dash patterns
    #[param(slider, logarithmic, min = 0.01, max = 10.0)]
    dash_1: Length,
    #[param(slider, logarithmic, min = 0.01, max = 10.0)]
    dash_2: Length,
}

impl Default for DashedLinesSketch {
    fn default() -> Self {
        Self {
            offset: 0.0 * Unit::Mm,
            dash_1: 0.5 * Unit::Mm,
            dash_2: 1.0 * Unit::Mm,
        }
    }
}

impl App for DashedLinesSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        self.add_dashed(
            sketch,
            kurbo::Line::new((50.0, 50.0), (sketch.width() - 50.0, 50.0)),
        );

        self.add_dashed(
            sketch,
            kurbo::Ellipse::new(
                (sketch.width() / 2.0, 200.),
                (sketch.width() / 2.0 - 50., 50.0),
                0.,
            ),
        );

        Ok(())
    }
}

impl DashedLinesSketch {
    fn add_dashed(&self, sketch: &mut Sketch, path: impl IntoBezPath) {
        let path = path.into_bezpath();

        // draw path endings
        if let Some(start) = path.start() {
            sketch.circle(start.x(), start.y(), 1.0);
        }
        if let Some(end) = path.end() {
            sketch.circle(end.x(), end.y(), 1.0);
        }

        let dashed: kurbo::BezPath = kurbo::dash(
            path.into_bezpath().into_iter(),
            self.offset.to_px(),
            &[self.dash_1.to_px(), self.dash_2.to_px()],
        )
        .collect();

        sketch.add_path(dashed);
    }
}

fn main() -> Result {
    DashedLinesSketch::runner().run()
}
