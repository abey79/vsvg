use kurbo::PathEl;

use std::error::Error;
use std::fs;
use std::path;

use crate::types::{Color, Document, Layer, PageSize, Path};

use usvg::utils::view_box_to_transform;
use usvg::{PathSegment, Transform};

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

impl Document {
    /// Create a `Document` based on a path to an SVG file.
    pub fn from_svg<P: AsRef<path::Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let svg = fs::read_to_string(path)?;
        Document::from_string(&svg)
    }

    /// Create a `Document` based on a string containing SVG data.
    pub fn from_string(svg: &str) -> Result<Self, Box<dyn Error>> {
        let tree = usvg::Tree::from_str(svg, &usvg::Options::default())?;

        let viewbox_transform =
            view_box_to_transform(tree.view_box.rect, tree.view_box.aspect, tree.size);

        // add frame for the page
        let (w, h) = (tree.size.width(), tree.size.height());
        let mut doc = Document::new_with_page_size(PageSize { w, h });

        let mut top_level = Layer::new();
        for child in tree.root.children() {
            let mut transform = viewbox_transform;
            transform.append(&child.borrow().transform());

            match *child.borrow() {
                usvg::NodeKind::Group(_) => {
                    doc.layers.push(parse_group(&child, &transform));
                }
                usvg::NodeKind::Path(ref path) => {
                    top_level.paths.push(Path::from_svg(path, &transform));
                }
                _ => {}
            }
        }

        // insert top-level path in the first layer
        if !top_level.paths.is_empty() {
            if let Some(layer) = doc.layers.first_mut() {
                layer.paths.append(&mut top_level.paths);
            } else {
                doc.layers.push(top_level);
            }
        }

        let doc = doc.crop(0., 0., w, h);

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_file;
    use crate::types::Document;
    use kurbo::BezPath;

    #[test]
    fn test_top_level_path_in_first_layer() {
        let doc = Document::from_svg(test_file!("multilayer.svg")).unwrap();
        assert_eq!(doc.layers.len(), 2);
        assert_eq!(doc.layers[0].paths.len(), 2);
        assert_eq!(doc.layers[1].paths.len(), 2);
    }

    #[test]
    fn test_viewbox() {
        let doc = Document::from_string(
            "<?xml version=\"1.0\"?>
            <svg xmlns:xlink=\"http://www.w3.org/1999/xlink\" xmlns=\"http://www.w3.org/2000/svg\"
               width=\"100\" height=\"100\" viewBox=\"50 50 10 10\">
               <line x1=\"50\" y1=\"50\" x2=\"60\" y2=\"60\" />
            </svg>",
        )
        .unwrap();

        let page_size = doc.page_size.unwrap();
        assert_eq!(page_size.w, 100.);
        assert_eq!(page_size.h, 100.);
        assert_eq!(doc.layers[0].paths.len(), 1);
        assert_eq!(
            doc.layers[0].paths[0].data,
            BezPath::from_svg("M 0 0 L 100 100").unwrap()
        );
    }
}
