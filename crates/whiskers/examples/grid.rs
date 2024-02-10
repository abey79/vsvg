//! This example demonstrates the use of the [`Grid`] helper.

use whiskers::prelude::*;

#[sketch_app]
struct GridSketch {
    #[param(slider, min = 20.0, max = 400.0)]
    width: f64,
    #[param(slider, min = 20.0, max = 400.0)]
    height: f64,
    #[param(slider, min = 2, max = 20)]
    columns: usize,
    #[param(slider, min = 2, max = 20)]
    rows: usize,
    #[param(slider, min = 0.0, max = 200.0)]
    gutter_width: f64,
    #[param(slider, min = 0.0, max = 200.0)]
    gutter_height: f64,
    is_canvas_sizing: bool,

    #[param(min = 0, max = self.columns - 1)]
    marked_cell_col: usize,
    #[param(min = 0, max = self.rows - 1)]
    marked_cell_row: usize,
}

impl Default for GridSketch {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 100.0,
            columns: 5,
            rows: 5,
            gutter_width: 20.0,
            gutter_height: 20.0,
            is_canvas_sizing: false,
            marked_cell_col: 0,
            marked_cell_row: 0,
        }
    }
}

impl App for GridSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.stroke_width(5.0);

        let grid = if self.is_canvas_sizing {
            Grid::from_total_size([sketch.width(), sketch.height()])
        } else {
            Grid::from_cell_size([self.width, self.height])
        };

        grid.columns(self.columns)
            .rows(self.rows)
            .spacing([self.gutter_width, self.gutter_height])
            .build(sketch, |sketch, cell| {
                sketch.color(
                    if cell.row == self.marked_cell_row && cell.column == self.marked_cell_col {
                        Color::GREEN
                    } else {
                        Color::RED
                    },
                );

                // when added to a sketch, a [`GridCell`] draws a rectangle at its location
                sketch.add_path(cell);
            });

        Ok(())
    }
}

fn main() -> Result {
    GridSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .with_layout_options(LayoutOptions::Center)
        .run()
}
