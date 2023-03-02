use kurbo::PathEl;

use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path;

use crate::{Color, Document, Layer, LayerID, PageSize, Path};

use usvg::utils::view_box_to_transform;
use usvg::{PathSegment, Transform};

impl Path {
    #[must_use]
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

fn parse_group(group: &usvg::Node, transform: &Transform, layer: &mut Layer) {
    group.children().for_each(|node| {
        let mut child_transform = *transform;
        child_transform.append(&node.borrow().transform());

        match *node.borrow() {
            usvg::NodeKind::Path(ref path) => {
                layer.paths.push(Path::from_svg(path, &child_transform));
            }
            usvg::NodeKind::Group(_) => {
                parse_group(&node, &child_transform, layer);
            }
            _ => {}
        }
    });
}

lazy_static! {
    static ref DIGITS_RE: Regex = Regex::new(r"\d+").unwrap();
}

/// Interpret the attributes of a top-level group to determine its layer ID.
///
/// See <https://github.com/abey79/vsvg/issues/7> for the strategy used here.
fn layer_id_from_attribute(attributes: &svg::node::Attributes) -> Option<LayerID> {
    fn extract_id(id: &str) -> Option<LayerID> {
        DIGITS_RE.find(id).map(|m| {
            let mut id = m
                .as_str()
                .parse::<usize>()
                .expect("regex guarantees only digits");
            if id == 0 {
                id = 1;
            }

            id
        })
    }

    if let Some(id) = attributes.get("inkscape:label") {
        let lid = extract_id(id);
        if lid.is_some() {
            return lid;
        }
    }

    if let Some(id) = attributes.get("id") {
        let lid = extract_id(id);
        if lid.is_some() {
            return lid;
        }
    }

    None
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

        // usvg doesn't give us access to original attributes, which we need to access
        // `inkscape:label` and `inkscape:groupmode`. As a work-around, we must use `svg` and
        // double-parse the SVG. See https://github.com/abey79/vsvg/issues/6
        // TODO: consider using roxmltree instead, to avoid double-parsing
        let mut nest_level = 0;
        let top_level_groups: Vec<_> = svg::read(svg)?
            .filter_map(|event| match event {
                svg::parser::Event::Tag(svg::node::element::tag::Group, tag_type, attributes) => {
                    match tag_type {
                        svg::node::element::tag::Type::Start => {
                            nest_level += 1;
                            if nest_level == 1 {
                                Some(attributes)
                            } else {
                                None
                            }
                        }
                        svg::node::element::tag::Type::End => {
                            nest_level -= 1;
                            None
                        }
                        svg::node::element::tag::Type::Empty => None,
                    }
                }
                _ => None,
            })
            .collect();

        let mut top_level_index = 0;
        for child in tree.root.children() {
            let mut transform = viewbox_transform;
            transform.append(&child.borrow().transform());

            match *child.borrow() {
                usvg::NodeKind::Group(_) => {
                    let attributes = &top_level_groups[top_level_index];
                    let id = layer_id_from_attribute(attributes).unwrap_or(top_level_index + 1);
                    top_level_index += 1;
                    parse_group(&child, &transform, doc.get_mut(id));

                    // set layer name
                    if let Some(name) = attributes.get("inkscape:label") {
                        doc.get_mut(id).name = name.to_string();
                    }
                }
                usvg::NodeKind::Path(ref path) => {
                    doc.get_mut(0).paths.push(Path::from_svg(path, &transform));
                }
                _ => {}
            }
        }

        Ok(doc.crop(0., 0., w, h))
    }
}

#[cfg(test)]
mod tests {

    use crate::{test_file, Document};
    use kurbo::BezPath;

    #[test]
    fn test_top_level_path_in_first_layer() {
        let doc = Document::from_svg(test_file!("multilayer.svg")).unwrap();
        assert_eq!(doc.layers.len(), 3);
        assert_eq!(doc.try_get(0).unwrap().paths.len(), 1);
        assert_eq!(doc.try_get(2).unwrap().paths.len(), 2);
        assert_eq!(doc.try_get(3).unwrap().paths.len(), 1);
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
        assert_eq!(doc.try_get(0).unwrap().paths.len(), 1);
        assert_eq!(
            doc.try_get(0).unwrap().paths[0].data,
            BezPath::from_svg("M 0 0 L 100 100").unwrap()
        );
    }
}
