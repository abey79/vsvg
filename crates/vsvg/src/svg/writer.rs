// design notes: https://github.com/abey79/vsvg/issues/13
// TODO: this is an initial take. Most of the spec is not implemented yet.

use svg::Node;

use crate::{DocumentTrait, LayerTrait, PathDataTrait, PathMetadata, PathTrait, Polyline};

// private trait in public types, see https://github.com/rust-lang/rust/issues/34537

pub trait SvgPathWriter {
    fn to_svg_path_data(&self) -> svg::node::element::path::Data;
}

impl SvgPathWriter for kurbo::BezPath {
    fn to_svg_path_data(&self) -> svg::node::element::path::Data {
        let mut data = svg::node::element::path::Data::new();

        for el in self.elements() {
            match el {
                kurbo::PathEl::MoveTo(pt) => data = data.move_to((pt.x, pt.y)),
                kurbo::PathEl::LineTo(pt) => data = data.line_to((pt.x, pt.y)),
                kurbo::PathEl::QuadTo(pt1, pt2) => {
                    data = data.quadratic_curve_to((pt1.x, pt1.y, pt2.x, pt2.y));
                }
                kurbo::PathEl::CurveTo(pt1, pt2, pt3) => {
                    data = data.cubic_curve_to((pt1.x, pt1.y, pt2.x, pt2.y, pt3.x, pt3.y));
                }
                kurbo::PathEl::ClosePath => data = data.close(),
            }
        }

        data
    }
}

impl SvgPathWriter for Polyline {
    fn to_svg_path_data(&self) -> svg::node::element::path::Data {
        let mut data = svg::node::element::path::Data::new();

        if self.points().len() < 2 {
            return data;
        }

        for (i, pt) in self.points().iter().enumerate() {
            if i == 0 {
                data = data.move_to((pt.x(), pt.y()));
            } else if i == self.points().len() - 1 && self.points().first() == self.points().last()
            {
                data = data.close();
            } else {
                data = data.line_to((pt.x(), pt.y()));
            }
        }

        data
    }
}

/// Apply stroke attributes from metadata to a Path element, only if they are Some.
fn apply_stroke_attrs_to_path(
    mut elem: svg::node::element::Path,
    metadata: &PathMetadata,
) -> svg::node::element::Path {
    if let Some(color) = metadata.color {
        elem = elem.set("stroke", color.to_rgb_string());
        if color.opacity() < 1.0 {
            elem = elem.set("stroke-opacity", format!("{:0.1}%", color.opacity() * 100.));
        }
    }
    if let Some(width) = metadata.stroke_width {
        elem = elem.set("stroke-width", width);
    }
    elem
}

/// Apply stroke attributes from metadata to a Group element, only if they are Some.
fn apply_stroke_attrs_to_group(
    mut elem: svg::node::element::Group,
    metadata: &PathMetadata,
) -> svg::node::element::Group {
    if let Some(color) = metadata.color {
        elem = elem.set("stroke", color.to_rgb_string());
        if color.opacity() < 1.0 {
            elem = elem.set("stroke-opacity", format!("{:0.1}%", color.opacity() * 100.));
        }
    }
    if let Some(width) = metadata.stroke_width {
        elem = elem.set("stroke-width", width);
    }
    elem
}

fn path_to_svg_path<P: PathTrait<D>, D: PathDataTrait>(path: &P) -> svg::node::element::Path {
    let mut elem = svg::node::element::Path::new()
        .set("fill", "none")
        .set("d", path.data().to_svg_path_data());

    // Only write path-level metadata if set (non-None)
    elem = apply_stroke_attrs_to_path(elem, path.metadata());

    elem
}

fn layer_to_svg_group<L: LayerTrait<P, D>, P: PathTrait<D>, D: PathDataTrait + SvgPathWriter>(
    layer: &L,
) -> svg::node::element::Group {
    let mut group = svg::node::element::Group::new()
        .set("inkscape:groupmode", "layer")
        .set("fill", "none");

    if let Some(name) = &layer.metadata().name {
        group = group.set("inkscape:label", name.as_str());
    }

    // Write layer defaults on <g> element
    let layer_defaults = &layer.metadata().default_path_metadata;
    group = apply_stroke_attrs_to_group(group, layer_defaults);

    // Paths only get their own overrides (non-None values)
    for path in layer.paths() {
        group = group.add(path_to_svg_path(path));
    }

    group
}

pub(crate) fn document_to_svg_doc<
    T: DocumentTrait<L, P, D>,
    L: LayerTrait<P, D>,
    P: PathTrait<D>,
    D: PathDataTrait,
>(
    document: &T,
) -> svg::Document {
    let mut doc = svg::Document::new()
        .set(
            "xmlns:inkscape",
            "http://www.inkscape.org/namespaces/inkscape",
        )
        .set("xmlns:cc", "http://creativecommons.org/ns")
        .set("xmlns:dc", "http://purl.org/dc/elements/1.1/")
        .set("xmlns:rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns");

    // dimensions, ensuring minimum size of 1x1
    let mut dims = if let Some(page_size) = document.metadata().page_size {
        kurbo::Rect::from_points((0.0, 0.0), page_size.to_pixels())
    } else if let Some(bounds) = document.bounds() {
        bounds
    } else {
        kurbo::Rect::from_points((0.0, 0.0), (1.0, 1.0))
    };
    dims = dims.union(kurbo::Rect::from_origin_size(dims.origin(), (1.0, 1.0)));

    doc = doc
        .set("width", format!("{:.5}", dims.width()))
        .set("height", format!("{:.5}", dims.height()))
        .set(
            "viewBox",
            format!(
                "{:.5} {:.5} {:.5} {:.5}",
                dims.x0,
                dims.y0,
                dims.width(),
                dims.height()
            ),
        );

    // append metadata
    let mut cc = svg::node::element::Element::new("cc:Work");
    let mut dc_format = svg::node::element::Element::new("dc:format");
    dc_format.append(svg::node::Text::new("image/svg+xml"));
    cc.append(dc_format);

    //TODO: find suitable replacement
    #[cfg(not(target_arch = "wasm32"))]
    if document.metadata().include_date {
        use time::OffsetDateTime;
        use time::format_description::well_known::Iso8601;

        let mut dc_date = svg::node::element::Element::new("dc:date");
        dc_date.append(svg::node::Text::new(
            OffsetDateTime::now_utc()
                .format(&Iso8601::DEFAULT)
                .expect("must format"),
        ));
        cc.append(dc_date);
    }

    if let Some(source) = document.metadata().source.as_ref() {
        let mut dc_source = svg::node::element::Element::new("dc:source");
        // Note: svg::node::Text automatically escapes special characters
        dc_source.append(svg::node::Text::new(source));
        cc.append(dc_source);
    }
    let mut rdf = svg::node::element::Element::new("rdf:RDF");
    rdf.append(cc);
    let mut metadata = svg::node::element::Element::new("metadata");
    metadata.append(rdf);
    doc.append(metadata);

    // append layers
    for (lid, layer) in document.layers() {
        let group = layer_to_svg_group(layer).set("id", format!("layer{lid}"));

        doc = doc.add(group);
    }

    doc
}

#[cfg(test)]
mod tests {
    use crate::{Color, Document, DocumentTrait, LayerTrait, PathTrait};

    #[test]
    fn test_svg_out() {
        let doc = Document::from_string(
            r#"<?xml version="1.0"?>
            <svg xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
               xmlns="http://www.w3.org/2000/svg"
               width="100" height="100" >
                <path d="M 10,0 L 20,0" />
            </svg>"#,
            false,
        )
        .unwrap();

        let svg = doc.to_svg_string().unwrap();

        assert!(svg.contains("path d=\"M10,0 L20,0\""));
    }

    #[test]
    fn test_svg_source_escape() {
        let mut doc = Document::default();
        doc.metadata_mut().source = Some("<hello>".to_owned());

        let svg = doc.to_svg_string().unwrap();
        assert!(svg.contains("&lt;hello&gt;"));
        assert!(!svg.contains("<hello>"));
    }

    #[test]
    fn test_svg_write_layer_defaults_on_group() {
        let mut doc = Document::default();
        let layer = doc.get_mut(1);

        // Set layer defaults
        layer.metadata_mut().default_path_metadata.color = Some(Color::RED);
        layer.metadata_mut().default_path_metadata.stroke_width = Some(2.5);

        // Add a path with no overrides
        layer.push_path(vec![(0., 0.), (10., 10.)]);

        let svg = doc.to_svg_string().unwrap();

        // Layer defaults should be on <g>
        assert!(
            svg.contains(r##"stroke="#ff0000""##),
            "group should have stroke color"
        );
        assert!(
            svg.contains(r#"stroke-width="2.5""#),
            "group should have stroke-width"
        );

        // Path should NOT have stroke attrs (they inherit from group)
        // Extract just the path element
        let path_start = svg.find("<path").unwrap();
        let path_end = svg[path_start..].find("/>").unwrap() + path_start + 2;
        let path_elem = &svg[path_start..path_end];
        assert!(
            !path_elem.contains("stroke="),
            "path should not have stroke attr"
        );
        assert!(
            !path_elem.contains("stroke-width"),
            "path should not have stroke-width attr"
        );
    }

    #[test]
    fn test_svg_write_path_overrides() {
        let mut doc = Document::default();
        let layer = doc.get_mut(1);

        // Set layer defaults
        layer.metadata_mut().default_path_metadata.color = Some(Color::RED);
        layer.metadata_mut().default_path_metadata.stroke_width = Some(2.0);

        // Add a path that overrides color but not stroke_width
        let mut path = crate::Path::from(vec![(0., 0.), (10., 10.)]);
        path.metadata_mut().color = Some(Color::BLUE);
        layer.push_path(path);

        let svg = doc.to_svg_string().unwrap();

        // Group should have layer defaults
        assert!(
            svg.contains(r##"stroke="#ff0000""##),
            "group should have red stroke"
        );

        // Path should only have its override (blue color)
        assert!(
            svg.contains(r##"stroke="#0000ff""##),
            "path should have blue stroke override"
        );
    }

    #[test]
    fn test_svg_write_none_metadata_not_written() {
        let mut doc = Document::default();
        let layer = doc.get_mut(1);

        // Layer has no defaults set (all None)
        // Path has no metadata set (all None)
        layer.push_path(vec![(0., 0.), (10., 10.)]);

        let svg = doc.to_svg_string().unwrap();

        // Should not contain stroke or stroke-width on either group or path
        // (they should inherit from SVG defaults)
        let group_part = svg.split("<path").next().unwrap();
        assert!(
            !group_part.contains("stroke="),
            "group should not have stroke when None"
        );
        assert!(
            !group_part.contains("stroke-width"),
            "group should not have stroke-width when None"
        );
    }
}
