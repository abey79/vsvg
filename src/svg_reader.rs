use std::error::Error;
use std::fs;
use std::path::Path;
use usvg::{NodeExt, PathSegment, Transform};

#[derive(Default)]
pub(crate) struct Line {
    pub points: Vec<[f64; 2]>,
}

impl From<Vec<[f64; 2]>> for Line {
    fn from(points: Vec<[f64; 2]>) -> Self {
        Self { points }
    }
}

#[derive(Default)]
pub(crate) struct Lines {
    pub lines: Vec<Line>,
}

impl Lines {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }

    fn add_lines(&mut self, lines: Vec<Line>) {
        self.lines.extend(lines);
    }
}

fn path_to_plot_points(path: &usvg::Path, transform: &usvg::Transform) -> Vec<Line> {
    let mut output: Vec<Line> = vec![];
    let mut line = vec![];
    for elem in usvg::TransformedPath::new(&path.data, *transform) {
        match elem {
            PathSegment::MoveTo { x, y } => {
                if !line.is_empty() {
                    output.push(Line::from(line));
                    line = Vec::new();
                }
                line.push([x, y]);
            }
            PathSegment::LineTo { x, y } => line.push([x, y]),
            PathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                // todo: hardcoded to 10 points
                const N: usize = 10;
                let first = *line.last().unwrap();
                line.extend((1..N).map(move |i| {
                    let t = i as f64 / (N - 1) as f64;
                    let ttt = t * t * t;
                    let ttu = t * t * (1. - t);
                    let tuu = t * (1. - t) * (1. - t);
                    let uuu = (1. - t) * (1. - t) * (1. - t);

                    [
                        first[0] * uuu + 3. * x1 * tuu + 3. * x2 * ttu + x * ttt,
                        first[1] * uuu + 3. * y1 * tuu + 3. * y2 * ttu + y * ttt,
                    ]
                }));
            }
            PathSegment::ClosePath => line.push(line[0]),
        }
    }
    if !line.is_empty() {
        output.push(Line::from(line));
    }

    output
}

fn parse_group(group: &usvg::Node, transform: &usvg::Transform) -> Vec<Line> {
    group
        .children()
        .flat_map(|node| {
            let mut child_transform = *transform;
            child_transform.append(&node.borrow().transform());

            match *node.borrow() {
                usvg::NodeKind::Path(ref path) => path_to_plot_points(path, &child_transform),
                usvg::NodeKind::Group(_) => parse_group(&node, &child_transform),
                _ => {
                    vec![]
                }
            }
        })
        .collect()
}

pub(crate) fn parse_svg<P: AsRef<Path>>(path: P) -> Result<Lines, Box<dyn Error>> {
    let svg = fs::read_to_string(path)?;
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default())?;

    let mut output = Lines::new();

    // add frame for the page
    let (w, h) = (tree.size.width(), tree.size.height());
    output.lines.push(Line {
        points: vec![[0., 0.], [w, 0.], [w, h], [0., h], [0., 0.]],
    });

    // setup transform to account for egui's y-up setup.
    let mut global_transform = Transform::new_scale(1., -1.);
    global_transform.translate(0., -h);
    global_transform.append(&tree.root.transform());

    for child in tree.root.children() {
        if let usvg::NodeKind::Group(_) = *child.borrow() {
            let mut transform = global_transform;
            transform.append(&child.borrow().transform());
            output.add_lines(parse_group(&child, &transform));
        }
    }

    Ok(output)
}
