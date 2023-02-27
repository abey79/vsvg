use kurbo::PathEl;

use std::error::Error;
use std::fs;
use std::path;

use crate::types::{Color, Document, Layer, PageSize, Path};

use usvg::{NodeExt, PathSegment, Transform};

impl Path {
    pub fn from_svg(svg_path: &usvg::Path, transform: &Transform) -> Self {
        let bezpath = usvg::TransformedPath::new(&svg_path.data, *transform)
            .into_iter()
            .map(|elem| match elem {
                PathSegment::MoveTo { x, y } => PathEl::MoveTo(kurbo::Point::new(x, y)),
                PathSegment::LineTo { x, y } => PathEl::LineTo(kurbo::Point::new(x, y)),
                PathSegment::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => PathEl::CurveTo(
                    kurbo::Point::new(x1, y1),
                    kurbo::Point::new(x2, y2),
                    kurbo::Point::new(x, y),
                ),
                PathSegment::ClosePath => PathEl::ClosePath,
            })
            .collect();

        let mut res = Self {
            data: bezpath,
            ..Default::default()
        };

        // extract metadata
        if let Some(stroke) = &svg_path.stroke {
            if let usvg::Paint::Color(c) = stroke.paint {
                res.color = Color {
                    r: c.red,
                    g: c.green,
                    b: c.blue,
                    a: 255,
                };
            }
            res.stroke_width = stroke.width.get();
        }

        res
    }
}

fn parse_group(group: &usvg::Node, transform: &Transform) -> Layer {
    let mut layer = Layer::new();

    group.children().for_each(|node| {
        let mut child_transform = *transform;
        child_transform.append(&node.borrow().transform());

        match *node.borrow() {
            usvg::NodeKind::Path(ref path) => {
                layer.paths.push(Path::from_svg(path, &child_transform));
            }
            usvg::NodeKind::Group(_) => {
                let sub_layer = parse_group(&node, &child_transform);
                layer.paths.extend(sub_layer.paths);
            }
            _ => {}
        }
    });

    layer
}

pub(crate) fn parse_svg<P: AsRef<path::Path>>(path: P) -> Result<Document, Box<dyn Error>> {
    let svg = fs::read_to_string(path)?;
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default())?;

    // add frame for the page
    let (w, h) = (tree.size.width(), tree.size.height());
    let mut doc = Document::new_with_page_size(PageSize { w, h });

    for child in tree.root.children() {
        if let usvg::NodeKind::Group(_) = *child.borrow() {
            let mut transform = Transform::default();
            transform.append(&child.borrow().transform());
            doc.layers.push(parse_group(&child, &transform));
        }

        // TODO: we're missing top-level paths here!
    }

    let doc = doc.crop(0., 0., w, h);

    Ok(doc)
}
