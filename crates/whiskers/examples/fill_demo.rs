//! Demonstrates the fill/hatching feature with stroke and fill layer routing.
//!
//! Shows a grid of shapes (circle, square, non-convex polygon, polygon with holes)
//! across three rows: no fill, horizontal fill, and 45° fill.

use vsvg::LayerID;
use whiskers::prelude::*;

#[sketch_app]
struct FillDemoSketch {
    #[param(slider, min = 0.05, max = 4.0)]
    pen_width: Length,

    #[param(slider, min = 0.0, max = 180.0)]
    hatch_angle: f64,

    /// Use 85% pen width for display to inspect hatching pattern
    inspect_hatching: bool,
}

impl Default for FillDemoSketch {
    fn default() -> Self {
        Self {
            pen_width: 0.35 * Unit::Mm,
            hatch_angle: 45.0,
            inspect_hatching: false,
        }
    }
}

impl App for FillDemoSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        // Configure layers (both use same pen width):
        // - Layer 0: stroke outlines (black)
        // - Layer 1: hatching fill (dark red)
        sketch
            .layer(0)
            .pen_width(self.pen_width)
            .color(Color::BLACK)
            .name("Stroke");
        sketch
            .layer(1)
            .pen_width(self.pen_width)
            .color(Color::DARK_RED)
            .name("Fill");

        // Set stroke width for display
        let display_width = if self.inspect_hatching {
            self.pen_width * 0.85
        } else {
            self.pen_width
        };
        sketch.stroke_width(display_width);

        // Grid layout with margins
        let margin = 10.0;
        let cols = 4;
        let rows = 3;
        let grid_w = sketch.width() - 2.0 * margin;
        let grid_h = sketch.height() - 2.0 * margin;
        let cell_size = (grid_w / cols as f64).min(grid_h / rows as f64);
        let shape_size = cell_size * 0.7;

        // Center the grid
        let grid_total_w = cell_size * cols as f64;
        let grid_total_h = cell_size * rows as f64;
        let offset_x = margin + (grid_w - grid_total_w) / 2.0;
        let offset_y = margin + (grid_h - grid_total_h) / 2.0;

        // Row configurations: (fill_layer, hatch_angle)
        let row_configs: [(Option<LayerID>, f64); 3] = [
            (None, 0.0),                              // No fill
            (Some(1), 0.0),                           // Horizontal fill
            (Some(1), self.hatch_angle.to_radians()), // Angled fill
        ];

        for (row, (fill_layer, hatch_angle)) in row_configs.iter().enumerate() {
            let cy = offset_y + cell_size * (row as f64 + 0.5);

            // Configure fill for this row
            sketch.layer(1).hatch_angle(*hatch_angle);
            sketch.stroke_layer(Some(0)).fill_layer(*fill_layer);

            // Column 0: Circle
            let cx = offset_x + cell_size * 0.5;
            sketch.circle(cx, cy, shape_size / 2.0);

            // Column 1: Square
            let cx = offset_x + cell_size * 1.5;
            sketch.rect(cx, cy, shape_size, shape_size);

            // Column 2: Non-convex polygon (L-shape)
            let cx = offset_x + cell_size * 2.5;
            let s = shape_size / 2.0;
            sketch.polyline(
                [
                    (cx - s, cy - s),
                    (cx + s, cy - s),
                    (cx + s, cy),
                    (cx, cy),
                    (cx, cy + s),
                    (cx - s, cy + s),
                ],
                true,
            );

            // Column 3: Polygon with 2 holes
            let cx = offset_x + cell_size * 3.5;
            let s = shape_size / 2.0;

            // Create a geo polygon with holes
            let exterior = geo::LineString::from(vec![
                (cx - s, cy - s),
                (cx + s, cy - s),
                (cx + s, cy + s),
                (cx - s, cy + s),
                (cx - s, cy - s),
            ]);

            let hole_size = s * 0.3;
            let hole1 = geo::LineString::from(vec![
                (cx - s * 0.5 - hole_size, cy - hole_size),
                (cx - s * 0.5 + hole_size, cy - hole_size),
                (cx - s * 0.5 + hole_size, cy + hole_size),
                (cx - s * 0.5 - hole_size, cy + hole_size),
                (cx - s * 0.5 - hole_size, cy - hole_size),
            ]);

            let hole2 = geo::LineString::from(vec![
                (cx + s * 0.5 - hole_size, cy - hole_size),
                (cx + s * 0.5 + hole_size, cy - hole_size),
                (cx + s * 0.5 + hole_size, cy + hole_size),
                (cx + s * 0.5 - hole_size, cy + hole_size),
                (cx + s * 0.5 - hole_size, cy - hole_size),
            ]);

            let polygon_with_holes = geo::Polygon::new(exterior, vec![hole1, hole2]);
            sketch.add_path(polygon_with_holes);
        }

        Ok(())
    }
}

fn main() -> Result {
    FillDemoSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .run()
}
