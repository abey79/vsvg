use whiskers::grid::cell::GridCell;
use whiskers::{grid::Grid, prelude::*};

#[derive(Sketch)]
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
    default_draw_function_enabled: bool,
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
            default_draw_function_enabled: false,
        }
    }
}

impl App for GridSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.stroke_width(5.0);

        fn fill_grid<'a>(column: usize, row: usize, _: &'a Vec<GridCell<Color>>) -> Option<Color> {
            let grey = ((row as f64) * (column as f64)).cos() * 100.0;
            Some(Color::rgb(80, grey as u8, 120))
        }

        let mut grid = Grid::<Color>::new(
            sketch,
            self.columns,
            self.rows,
            if self.is_canvas_sizing {
                grid::GridSize::GridBased([sketch.width(), sketch.height()])
            } else {
                grid::GridSize::CellBased([self.width, self.height])
            },
            [self.gutter_width, self.gutter_height],
            Point::new(0.0, 0.0),
        );

        grid.init(Some(fill_grid));

        if self.default_draw_function_enabled {
            grid.draw();
        } else {
            grid.data.iter().for_each(|cell| {
                sketch.color(cell.data.unwrap_or(Color::DARK_GRAY));
                sketch.rect(
                    cell.canvas_position.x() + (cell.size[0] / 2.0),
                    cell.canvas_position.y() + (cell.size[1] / 2.0),
                    cell.size[0],
                    cell.size[1],
                );
            });
        }

        Ok(())
    }
}

fn main() -> Result {
    Runner::new(GridSketch::default())
        .with_page_size_options(PageSize::A5H)
        .run()
}
