use kurbo::PathEl;

use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path;

use crate::{Color, Document, Layer, LayerID, PageSize, Path};

use usvg::utils::view_box_to_transform;
use usvg::{GroupMode, PathSegment, Transform};

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
fn layer_id_from_attribute(id: &str, label: Option<&str>) -> Option<LayerID> {
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

    if let Some(id) = label {
        let lid = extract_id(id);
        if lid.is_some() {
            return lid;
        }
    }

    extract_id(id)
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

        let mut top_level_index = 0;
        for child in tree.root.children() {
            let mut transform = viewbox_transform;
            transform.append(&child.borrow().transform());

            match *child.borrow() {
                usvg::NodeKind::Group(ref group_info) => {
                    let layer_id;
                    let layer_name;
                    match group_info.mode {
                        // top-level group without layer information
                        GroupMode::Normal => {
                            top_level_index += 1;
                            layer_id = layer_id_from_attribute(&group_info.id, None);
                            layer_name = if group_info.id.is_empty() {
                                None
                            } else {
                                Some(&group_info.id)
                            }
                        }
                        // top-level group with inkscape layer information
                        GroupMode::Layer(ref label) => {
                            top_level_index += 1;
                            layer_id = layer_id_from_attribute(&group_info.id, Some(label));
                            layer_name = if !label.is_empty() {
                                Some(label)
                            } else if !group_info.id.is_empty() {
                                Some(&group_info.id)
                            } else {
                                None
                            }
                        }
                        // this is a top-level path that was embedded in a group by usvg
                        GroupMode::Virtual => {
                            layer_id = Some(0);
                            layer_name = None;
                        }
                    }

                    let layer = doc.get_mut(layer_id.unwrap_or(top_level_index));
                    parse_group(&child, &transform, layer);

                    // set layer name
                    if let Some(name) = layer_name {
                        layer.name = name.clone();
                    }
                }
                usvg::NodeKind::Path(ref path) => {
                    doc.get_mut(0).paths.push(Path::from_svg(path, &transform));
                }
                _ => {}
            }
        }

        doc.crop(0., 0., w, h);
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {

    use crate::{test_file, Document};
    use kurbo::BezPath;

    #[test]
    fn test_top_level_path_in_layer_0() {
        let doc = Document::from_svg(test_file!("multilayer.svg")).unwrap();
        assert_eq!(doc.layers.len(), 3);
        assert_eq!(doc.try_get(0).unwrap().paths.len(), 1);
        assert_eq!(doc.try_get(2).unwrap().paths.len(), 2);
        assert_eq!(doc.try_get(3).unwrap().paths.len(), 1);
    }

    #[test]
    fn test_single_layer() {
        let doc = Document::from_svg(test_file!("singlelayer.svg")).unwrap();
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.try_get(3).unwrap().paths.len(), 1);
    }

    #[test]
    fn test_virtual_group() {
        // this SVG triggers the creation of a virtual group by usvg, which should not be considered
        // a top-level group and should not be assigned a layer ID
        let doc = Document::from_svg(test_file!("spurious_group.svg")).unwrap();
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.try_get(0).unwrap().paths.len(), 2);
    }

    #[test]
    fn test_layer_names() {
        let doc = Document::from_string(
            r#"<?xml version="1.0"?>
            <svg xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
               xmlns="http://www.w3.org/2000/svg"
               width="100" height="100" viewBox="50 50 10 10">
                <g id="layer10" inkscape:label="Layer 10" inkscape:groupmode="layer">
                  <line x1="50" y1="50" x2="60" y2="60" />
                </g>
                <g id="layer11" >
                  <line x1="50" y1="50" x2="60" y2="60" />
                </g>
                <g inkscape:label="Hello" inkscape:groupmode="layer">
                  <line x1="50" y1="50" x2="60" y2="60" />
                </g>
                <g id="world">
                  <line x1="50" y1="50" x2="60" y2="60" />
                </g>
                <g id="notaname" inkscape:label="layer_name" inkscape:groupmode="layer">
                  <line x1="50" y1="50" x2="60" y2="60" />
                </g>
            </svg>"#,
        )
        .unwrap();

        assert_eq!(doc.layers.len(), 5);
        assert_eq!(doc.try_get(10).unwrap().name, "Layer 10");
        assert_eq!(doc.try_get(11).unwrap().name, "layer11");
        assert_eq!(doc.try_get(3).unwrap().name, "Hello");
        assert_eq!(doc.try_get(4).unwrap().name, "world");
        assert_eq!(doc.try_get(5).unwrap().name, "layer_name");
    }

    #[test]
    fn test_layer_numbering() {
        let doc = Document::from_string(
            r#"<?xml version="1.0"?>
            <svg xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
               xmlns="http://www.w3.org/2000/svg"
               width="100" height="100" viewBox="50 50 10 10">
                <g id="layer_one" >
                  <line x1="50" y1="50" x2="60" y2="60" />
                </g>
                <!-- this should trigger a virtual layer -->
                <line x1="50" y1="50" x2="60" y2="60" opacity="0.5" />
                <g id="layer11" >
                  <line x1="50" y1="50" x2="60" y2="60" />
                </g>
                <g id="layer_three" >
                  <line x1="50" y1="50" x2="60" y2="60" />
                </g>
            </svg>"#,
        )
        .unwrap();

        assert_eq!(doc.layers.len(), 4);
        assert_eq!(doc.try_get(0).unwrap().paths.len(), 1);
        assert_eq!(doc.try_get(1).unwrap().name, "layer_one");
        assert_eq!(doc.try_get(11).unwrap().name, "layer11");
        assert_eq!(doc.try_get(3).unwrap().name, "layer_three");
    }

    #[test]
    fn test_viewbox() {
        let doc = Document::from_string(
            r#"<?xml version="1.0"?>
            <svg xmlns:xlink="http://www.w3.org/1999/xlink" xmlns="http://www.w3.org/2000/svg"
               width="100" height="100" viewBox="50 50 10 10">
               <line x1="50" y1="50" x2="60" y2="60" />
            </svg>"#,
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
