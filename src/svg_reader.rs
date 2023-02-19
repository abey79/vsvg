use kurbo::{BezPath, PathEl};

use std::error::Error;
use std::fs;
use std::path;

use crate::types::Document;

use usvg::{NodeExt, PathSegment, Transform};

fn usvg_to_kurbo_path(path: &usvg::Path, transform: &Transform) -> BezPath {
    usvg::TransformedPath::new(&path.data, *transform)
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
        .collect()
}

fn parse_group(group: &usvg::Node, transform: &Transform) -> Vec<BezPath> {
    //TODO: we're not keeping the group structure here :)
    group
        .children()
        .flat_map(|node| {
            let mut child_transform = *transform;
            child_transform.append(&node.borrow().transform());

            match *node.borrow() {
                usvg::NodeKind::Path(ref path) => vec![usvg_to_kurbo_path(path, &child_transform)],
                usvg::NodeKind::Group(_) => parse_group(&node, &child_transform),
                _ => {
                    vec![]
                }
            }
        })
        .collect()
}

pub(crate) fn parse_svg<P: AsRef<path::Path>>(path: P) -> Result<Document, Box<dyn Error>> {
    let svg = fs::read_to_string(path)?;
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default())?;

    // add frame for the page
    let (w, h) = (tree.size.width(), tree.size.height());

    // setup transform to account for egui's y-up setup.
    let mut global_transform = Transform::new_scale(1., -1.);
    global_transform.translate(0., -h);
    global_transform.append(&tree.root.transform());

    let mut doc = Document::new();
    doc.add_path(kurbo::Rect::new(0., 0., w, h));

    for child in tree.root.children() {
        if let usvg::NodeKind::Group(_) = *child.borrow() {
            let mut transform = global_transform;
            transform.append(&child.borrow().transform());
            doc.add_paths(parse_group(&child, &transform));
        }
    }

    let doc = doc.crop(0., 0., w, h);

    Ok(doc)
}
