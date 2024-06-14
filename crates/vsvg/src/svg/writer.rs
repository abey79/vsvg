// design notes: https://github.com/abey79/vsvg/issues/13
// TODO: this is an initial take. Most of the spec is not implemented yet.

use svg::Node;

use crate::{DocumentTrait, LayerTrait, PathDataTrait, PathTrait, Polyline};

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

fn path_to_svg_path<P: PathTrait<D>, D: PathDataTrait>(path: &P) -> svg::node::element::Path {
    let mut elem = svg::node::element::Path::new()
        .set("fill", "none")
        .set("stroke", path.metadata().color.to_rgb_string())
        .set("stroke-width", path.metadata().stroke_width)
        .set("d", path.data().to_svg_path_data());

    if path.metadata().color.opacity() < 1.0 {
        elem = elem.set(
            "stroke-opacity",
            format!("{:0.1}%", path.metadata().color.opacity() * 100.),
        );
    }

    elem

    // TODO: do not add metadata if it is the default
    // TODO: promote common attributes to group level
}

fn layer_to_svg_group<L: LayerTrait<P, D>, P: PathTrait<D>, D: PathDataTrait + SvgPathWriter>(
    layer: &L,
) -> svg::node::element::Group {
    let mut group = svg::node::element::Group::new().set("inkscape:groupmode", "layer");

    if let Some(name) = &layer.metadata().name {
        group = group.set("inkscape:label", name.as_str());
    }

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
    {
        use time::format_description::well_known::Iso8601;
        use time::OffsetDateTime;

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
        dc_source.append(svg::node::Text::new(quick_xml::escape::escape(source)));
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
    use super::*;
    use crate::Document;

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
    fn test_svg_source_escale() {
        let mut doc = Document::default();
        doc.metadata_mut().source = Some("<hello>".to_owned());

        let svg = doc.to_svg_string().unwrap();
        assert!(svg.contains("&lt;hello&gt;"));
        assert!(!svg.contains("<hello>"));
    }
}
