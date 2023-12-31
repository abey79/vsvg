//! This example demonstrates the use of the [`HexGrid`] helper.
use whiskers::prelude::*;

#[sketch_app]
struct HexGridSketch {
    is_pointy_orientation: bool,
    #[param(slider, min = 2, max = 20)]
    columns: usize,
    #[param(slider, min = 2, max = 20)]
    rows: usize,
    #[param(slider, min = 0.0, max = 200.0)]
    spacing: f64,
    #[param(min = 0.0, max = 100.0)]
    cell_size: f64,
}

impl Default for HexGridSketch {
    fn default() -> Self {
        Self {
            columns: 5,
            rows: 5,
            spacing: 0.0,
            is_pointy_orientation: false,
            cell_size: 40.0,
        }
    }
}

impl App for HexGridSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.stroke_width(5.0);

        let grid = if self.is_pointy_orientation {
            HexGrid::with_pointy_orientation()
        } else {
            HexGrid::with_flat_orientation()
        };

        grid.cell_size(self.cell_size)
            .columns(self.columns)
            .rows(self.rows)
            .spacing(self.spacing)
            .build(sketch, |sketch, cell| {
                sketch.add_path(cell);
            });

        Ok(())
    }
}

fn main() -> Result {
    HexGridSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .with_layout_options(LayoutOptions::Center)
        .run()
}
