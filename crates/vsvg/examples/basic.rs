use vsvg::{DocumentTrait, LayerTrait, PathTrait};

fn main() {
    /* == Document == */
    let mut doc = vsvg::Document::default();

    // push a path to layer 1
    doc.push_path(1, vec![(0., 0.), (100., 100.), (200., 0.), (0., 0.)]);

    /* == Layers == */
    let mut layer = vsvg::Layer::default();
    layer.metadata_mut().name = Some("Layer 2".to_string());

    // vsvg uses kurbo internally, and its API is compatible with it
    layer.push_path(kurbo::Circle::new((50., 50.), 30.));
    doc.layers_mut().insert(2, layer);

    /* == Path == */
    // Amongst various ways to create a path, the SVG <path> syntax is supported.
    let mut path = vsvg::Path::from_svg("M 200 200 L 200 400 Q 500 300 200 200 Z").unwrap();
    path.metadata_mut().color = vsvg::Color::DARK_GREEN;
    path.metadata_mut().stroke_width = 3.0;
    doc.push_path(3, path);

    // save to SVG
    doc.to_svg_file("basic.svg").unwrap();
}
