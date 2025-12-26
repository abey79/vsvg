//! Demonstrates `FlattenedPath::fit_to_path` for fitting smooth Bézier curves to polylines.
//!
//! This example shows the round-trip: Path → flatten → FlattenedPath → fit_to_path → Path
//!
//! Several shapes are demonstrated in columns:
//! - Circle, Ellipse, Rectangle, Cubic Bézier, Line
//!
//! Three rows show the pipeline:
//! - Top: original paths
//! - Middle: flattened polylines
//! - Bottom: re-fitted Bézier curves

use whiskers::prelude::*;

#[sketch_app]
struct FitToBezpathSketch {
    #[param(slider, min = 0.01, max = 10.0, logarithmic)]
    flatten_tolerance: f64,

    #[param(slider, min = 0.01, max = 10.0, logarithmic)]
    fit_tolerance: f64,
}

impl Default for FitToBezpathSketch {
    fn default() -> Self {
        Self {
            flatten_tolerance: 1.0,
            fit_tolerance: 2.0,
        }
    }
}

impl App for FitToBezpathSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        let w = sketch.width();
        let h = sketch.height();

        let cols = 5;
        let rows = 3;
        let cell_w = w / cols as f64;
        let cell_h = h / rows as f64;
        let shape_size = cell_w.min(cell_h) * 0.7;

        // Create the original paths using kurbo shapes
        let shapes: Vec<vsvg::Path> = vec![
            // Circle
            vsvg::Path::from(kurbo::Circle::new((0.0, 0.0), shape_size / 2.0)),
            // Ellipse
            vsvg::Path::from(kurbo::Ellipse::new(
                (0.0, 0.0),
                (shape_size / 2.0, shape_size / 3.0),
                0.3,
            )),
            // Square
            vsvg::Path::from(kurbo::Rect::new(
                -shape_size / 2.0,
                -shape_size / 2.0,
                shape_size / 2.0,
                shape_size / 2.0,
            )),
            // Cubic Bézier
            vsvg::Path::from(kurbo::CubicBez::new(
                (-shape_size / 2.0, shape_size / 3.0),
                (-shape_size / 4.0, -shape_size / 2.0),
                (shape_size / 4.0, shape_size / 2.0),
                (shape_size / 2.0, -shape_size / 3.0),
            )),
            // Line
            vsvg::Path::from(kurbo::Line::new(
                (-shape_size / 2.0, shape_size / 4.0),
                (shape_size / 2.0, -shape_size / 4.0),
            )),
        ];

        for (col, path) in shapes.into_iter().enumerate() {
            let cx = cell_w * (col as f64 + 0.5);

            // Flatten the path
            let flattened_paths = path.flatten(self.flatten_tolerance);

            // Fit curves back
            let fitted_paths: Vec<_> = flattened_paths
                .iter()
                .map(|fp| fp.fit_to_path(self.fit_tolerance))
                .collect();

            // Row 0: Original
            sketch
                .push_matrix()
                .translate(cx, cell_h * 0.5)
                .color(Color::BLACK)
                .stroke_width(0.5)
                .add_path(path.data.clone())
                .pop_matrix();

            // Row 1: Flattened (as polylines)
            sketch
                .push_matrix()
                .translate(cx, cell_h * 1.5)
                .color(Color::BLUE)
                .stroke_width(0.5);

            for fp in &flattened_paths {
                sketch.add_path(fp.data.clone());
            }

            sketch.pop_matrix();

            // Row 2: Fitted (with original underneath)
            sketch
                .push_matrix()
                .translate(cx, cell_h * 2.5)
                .color(Color::LIGHT_GRAY)
                .stroke_width(4.0)
                .add_path(path.data.clone())
                .color(Color::RED)
                .stroke_width(0.5);

            for fitted in &fitted_paths {
                sketch.add_path(fitted.data.clone());
            }

            sketch.pop_matrix();
        }

        Ok(())
    }
}

fn main() -> Result {
    FitToBezpathSketch::runner()
        .with_page_size_options(PageSize::A4H)
        .with_layout_options(LayoutOptions::Center)
        .with_display_options(
            DisplayOptions::default()
                .with_show_points(true)
                .with_show_control_points(true),
        )
        .run()
}
