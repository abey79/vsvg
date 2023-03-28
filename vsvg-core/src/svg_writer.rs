// design notes: https://github.com/abey79/vsvg/issues/13
// TODO: this is an initial take. Most of the spec is not implemented yet.

use crate::{DocumentImpl, LayerImpl, PathImpl, PathType};
use std::fmt::Display;
use svg::Node;

use svg::node::element::{Element, Group, Path};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

// private trait in public types, see https://github.com/rust-lang/rust/issues/34537
mod detail {
    use crate::{PathData, Polyline};
    use kurbo::PathEl;
    use svg::node::element::path;

    pub trait SvgPathWriter {
        fn svg_path_data(&self) -> path::Data;
    }

    impl SvgPathWriter for PathData {
        fn svg_path_data(&self) -> path::Data {
            let mut data = path::Data::new();

            for el in self.elements() {
                match el {
                    PathEl::MoveTo(pt) => data = data.move_to((pt.x, pt.y)),
                    PathEl::LineTo(pt) => data = data.line_to((pt.x, pt.y)),
                    PathEl::QuadTo(pt1, pt2) => {
                        data = data.quadratic_curve_to((pt1.x, pt1.y, pt2.x, pt2.y));
                    }
                    PathEl::CurveTo(pt1, pt2, pt3) => {
                        data = data.cubic_curve_to((pt1.x, pt1.y, pt2.x, pt2.y, pt3.x, pt3.y));
                    }
                    PathEl::ClosePath => data = data.close(),
                }
            }

            data
        }
    }

    impl SvgPathWriter for Polyline {
        fn svg_path_data(&self) -> path::Data {
            let mut data = path::Data::new();

            if self.len() < 2 {
                return data;
            }

            for (i, pt) in self.iter().enumerate() {
                if i == 0 {
                    data = data.move_to((pt.x(), pt.y()));
                } else if i == self.len() - 1 && self.first() == self.last() {
                    data = data.close();
                } else {
                    data = data.line_to((pt.x(), pt.y()));
                }
            }

            data
        }
    }
}

impl<T: PathType + detail::SvgPathWriter> PathImpl<T> {
    fn as_svg_path(&self) -> Path {
        Path::new()
            .set("fill", "none")
            .set("stroke", self.color.to_string())
            .set("stroke-width", self.stroke_width)
            .set("d", self.data.svg_path_data())

        // TODO: add metadata
        // TODO: do not add metadata if it is the default
        // TODO: promote common attributes to group level
    }
}

impl<T: PathType + detail::SvgPathWriter> LayerImpl<T> {
    fn as_svg_group(&self) -> Group {
        let mut group = Group::new()
            .set("inkscape:groupmode", "layer")
            .set("inkscape:label", self.name.as_str());

        for path in &self.paths {
            group = group.add(path.as_svg_path());
        }

        group
    }
}

impl<T: PathType + detail::SvgPathWriter> Display for DocumentImpl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut doc = svg::Document::new()
            .set(
                "xmlns:inkscape",
                "http://www.inkscape.org/namespaces/inkscape",
            )
            .set("xmlns:cc", "http://creativecommons.org/ns")
            .set("xmlns:dc", "http://purl.org/dc/elements/1.1/")
            .set("xmlns:rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns");

        // dimensions, ensuring minimum size of 1x1
        let mut dims = if let Some(page_size) = self.page_size {
            kurbo::Rect::from_points((0.0, 0.0), (page_size.w, page_size.h))
        } else if let Some(bounds) = self.bounds() {
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
        let mut cc = Element::new("cc:Work");
        let mut dc_format = Element::new("dc:format");
        dc_format.append(svg::node::Text::new("image/svg+xml"));
        cc.append(dc_format);
        let mut dc_date = Element::new("dc:date");
        dc_date.append(svg::node::Text::new(
            OffsetDateTime::now_utc()
                .format(&Iso8601::DEFAULT)
                .expect("must format"),
        ));
        cc.append(dc_date);
        if let Some(source) = self.source.as_ref() {
            let mut dc_source = Element::new("dc:source");
            dc_source.append(svg::node::Text::new(source));
            cc.append(dc_source);
        }
        let mut rdf = Element::new("rdf:RDF");
        rdf.append(cc);
        let mut metadata = Element::new("metadata");
        metadata.append(rdf);
        doc.append(metadata);

        // append layers
        for (lid, layer) in &self.layers {
            let group = layer.as_svg_group().set("id", format!("layer{lid}"));

            doc = doc.add(group);
        }

        write!(f, "{doc}")
    }
}

// TODO: tests
