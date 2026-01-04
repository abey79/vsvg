//! Example demonstrating the hatching API for filling closed shapes with parallel lines.

use kurbo::Shape;
use vsvg::{Color, DocumentTrait, FlattenedPath, HatchParams, Path, PathTrait, Unit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = vsvg::Document::default();

    let mut add_outline_and_hatches = |mut outline: Path, hatched: Vec<FlattenedPath>| {
        outline.metadata_mut().color = Color::BLACK;
        doc.push_path(0, outline);
        for mut path in hatched {
            path.metadata_mut().color = Color::RED;
            doc.push_path(1, path);
        }
    };

    // Create a circle and hatch it with horizontal lines
    let circle = Path::from(kurbo::Circle::new((100.0, 100.0), 50.0));
    let params = HatchParams::new(1.0 * Unit::Mm);
    let hatched = circle.hatch(&params, 0.1)?;
    add_outline_and_hatches(circle, hatched);

    // Create a square with diagonal hatching (45 degrees)
    let square = Path::from_svg("M 200,50 L 300,50 L 300,150 L 200,150 Z")?;
    let params = HatchParams::new(1.0 * Unit::Mm).with_angle(std::f64::consts::FRAC_PI_4);
    let hatched = square.hatch(&params, 0.1)?;
    add_outline_and_hatches(square, hatched);

    // Create a shape with a hole using kurbo shapes
    let mut donut_path = kurbo::BezPath::new();
    donut_path.extend(kurbo::Circle::new((400.0, 100.0), 60.0).path_elements(0.1));
    donut_path.extend(kurbo::Circle::new((400.0, 100.0), 25.0).path_elements(0.1));
    let donut = Path::from(donut_path);

    let params = HatchParams::new(1.0 * Unit::Mm)
        .with_angle(std::f64::consts::FRAC_PI_6) // 30 degrees
        .with_inset(true); // Include boundary stroke (default)
    let hatched = donut.hatch(&params, 0.1)?;

    add_outline_and_hatches(donut, hatched);

    // Demonstrate hatching without inset (lines extend to exact boundary)
    let triangle = Path::from_svg("M 500,150 L 600,150 L 550,50 Z")?;
    let params = HatchParams::new(1.0 * Unit::Mm)
        .with_inset(false) // No boundary inset
        .with_join_lines(false); // Keep lines separate (no joining)
    let hatched = triangle.hatch(&params, 0.1)?;

    add_outline_and_hatches(triangle, hatched);

    doc.to_svg_file("hatching.svg")?;

    Ok(())
}
