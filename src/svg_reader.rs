use std::error::Error;
use std::fs;
use std::path::Path;
use usvg::{NodeExt, PathSegment, Transform};

#[derive(Default)]
pub(crate) struct Lines {
    pub lines: Vec<Vec<[f64; 2]>>,
}

impl Lines {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }

    fn extend(&mut self, other: Self) {
        self.lines.extend(other.lines);
    }
}

fn path_to_plot_points(path: &usvg::Path, transform: &usvg::Transform) -> Lines {
    let mut output = Lines::new();
    let mut line = Vec::new();
    for elem in usvg::TransformedPath::new(&path.data, *transform) {
        println!("   {:?}", elem);
        match elem {
            PathSegment::MoveTo { x, y } => {
                if !line.is_empty() {
                    output.lines.push(line);
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
                let first = line[0];
                line.extend((0..N).map(move |i| {
                    let t = i as f64 / (N - 1) as f64;
                    let ttt = t * t * t;
                    let ttu = t * t * (1. - t);
                    let tuu = t * (1. - t) * (1. - t);
                    let uuu = (1. - t) * (1. - t) * (1. - t);

                    [
                        first[0] * ttt + 3. * x1 * ttu * 3. * x2 * tuu + x * uuu,
                        first[1] * ttt + 3. * y1 * ttu * 3. * y2 * tuu + y * uuu,
                    ]
                }));
            }
            PathSegment::ClosePath => line.push(line[0]),
        }
    }
    if !line.is_empty() {
        output.lines.push(line);
    }

    output
}

fn parse_group(group: &usvg::Node, &transform: &usvg::Transform) -> Lines {
    let mut output = Lines::new();

    for node in group.children() {
        let mut transform = transform.clone();
        transform.append(&node.borrow().transform());

        match *node.borrow() {
            usvg::NodeKind::Path(ref path) => {
                output.extend(path_to_plot_points(path, &transform));
            }
            usvg::NodeKind::Image(_) => {}
            usvg::NodeKind::Group(_) => {
                output.extend(parse_group(&node, &transform));
            }
            usvg::NodeKind::Text(_) => {}
        }
    }

    output
}

pub(crate) fn parse_svg<P: AsRef<Path>>(path: P) -> Result<Lines, Box<dyn Error>> {
    let svg = fs::read_to_string(path)?;
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default())?;

    let mut output = Lines::new();

    // add frame for the page
    let (w, h) = (tree.size.width(), tree.size.height());
    output
        .lines
        .push(vec![[0., 0.], [w, 0.], [w, h], [0., h], [0., 0.]]);

    // setup transform to account for egui's y-up setup.
    let mut global_transform = Transform::new_scale(1., -1.);
    global_transform.translate(0., -h);
    global_transform.append(&tree.root.transform());

    for child in tree.root.children() {
        if let usvg::NodeKind::Group(_) = *child.borrow() {
            let mut transform = global_transform.clone();
            transform.append(&child.borrow().transform());
            output.extend(parse_group(&child, &transform));
        }
    }

    Ok(output)
}
