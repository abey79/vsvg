use kurbo::{Point, Vec2};
use vsvg_core::Polyline;

pub type Triangle = (usize, usize, usize);

/// This function computes a triangulation to render fat lines.
pub fn build_fat_line(
    line: &Polyline,
    pen_width: f64,
    vertices: &mut Vec<Point>,
    triangles: &mut Vec<Triangle>,
) {
    let len = line.len();

    if len < 2 {
        //todo: handle len == 1 => two triangle for a single square
        return;
    }

    let mut push_v = |p| {
        vertices.push(p);
        vertices.len() - 1
    };

    let mut push_t = |i1, i2, i3| {
        triangles.push((i1, i2, i3));
    };

    // The strategy to handle closing lines is the following:
    // - generate the first two vertices as normal
    // - append line[1] at the end of the line iterator, so a full extra segment is
    //   generated (remember: line[0] is already the same as line[len - 1])
    // - copy the first two vertices from the last two vertices
    #[allow(clippy::float_cmp)]
    let closing = len > 3 && line[0] == line[len - 1];

    let w = pen_width / 2.0;

    let mut p1 = Point::new(line[0][0], line[0][1]);
    let mut p2 = Point::new(line[1][0], line[1][1]);

    let mut v1 = (p2 - p1).normalize();
    let mut n1 = Vec2 { x: -v1.y, y: v1.x };
    let mut critical_length_1 = (p2 - p1 + w * n1).hypot();

    // note: idx1 is always chosen to be on the side of the normal
    let mut idx1 = push_v(p1 + w * (-v1 + n1));
    let mut idx2 = push_v(p1 + w * (-v1 - n1));

    // remember those to close the loop
    let first_idx1 = idx1;
    let first_idx2 = idx2;

    let mut v0: Vec2;
    let mut n0: Vec2;
    let mut critical_length_0: f64;

    // if `closing`, the iterator has length len-1
    let iter = line[2..].iter().chain(if closing {
        line[1..2].iter()
    } else {
        [].iter()
    });
    let mut post_process_close = true;
    for (i, new_pt) in iter.enumerate() {
        // this is when we must "seam" the triangulation back to the first two vertices
        let finish_close = closing && i == len - 2;

        // p0 is where we're departing from, but not actually needed
        p1 = p2;
        p2 = Point::new(new_pt[0], new_pt[1]);

        v0 = v1;
        n0 = n1;
        v1 = (p2 - p1).normalize();
        n1 = Vec2 { x: -v1.y, y: v1.x };

        #[allow(clippy::float_cmp)]
        let turn_cw = Vec2::cross(v0, v1).signum() == 1.;
        let miter = (n0 + n1).normalize();
        let half_join = w / miter.dot(n0);

        critical_length_0 = critical_length_1;
        critical_length_1 = (p2 - p1 + w * n1).hypot();
        let restart = half_join >= critical_length_0 || half_join >= critical_length_1;

        if restart {
            // We interrupt the line here and restart a new one. This means that we must emit
            // two vertices at p1 and aligned with p0, then the two related triangles. Then we
            // must create two other vertices at p1, aligned with p2, ready for the next point.

            // In case we're closing and we must over-draw, we must emit two new closing
            // vertices, and related triangles, but skip creating new vertices for the next
            // point.

            let idx3 = push_v(p1 + w * (v0 + n0));
            let idx4 = push_v(p1 + w * (v0 - n0));
            push_t(idx1, idx2, idx3);
            push_t(idx2, idx3, idx4);

            if finish_close {
                // no need to adjust the first two vertices as we must accept the over-draw
                post_process_close = false;
            } else {
                // prepare for next line
                idx1 = push_v(p1 + w * (-v1 + n1));
                idx2 = push_v(p1 + w * (-v1 - n1));
            }
        } else {
            let idx3: usize;
            let idx4: usize;

            if Vec2::dot(v0, v1) >= 0. {
                // corner is less than 90° => no miter triangle is needed
                idx3 = push_v(p1 + half_join * miter);
                idx4 = push_v(p1 - half_join * miter);

                push_t(idx1, idx2, idx3);
                push_t(idx2, idx3, idx4);
            } else {
                // corner is more than 90° => miter triangle is needed
                // TBD: should the limit *really* be at 90°? Triangle count could be limited by
                // setting the threshold a bit higher...

                let idx5: usize;

                if turn_cw {
                    idx3 = push_v(p1 + half_join * miter);
                    idx4 = push_v(p1 + w * (-v1 - n1));
                    idx5 = push_v(p1 + w * (v0 - n0));
                    push_t(idx1, idx2, idx3);
                    push_t(idx2, idx3, idx5);
                } else {
                    idx3 = push_v(p1 + w * (-v1 + n1));
                    idx4 = push_v(p1 - half_join * miter);
                    idx5 = push_v(p1 + w * (v0 + n0));
                    push_t(idx1, idx2, idx5);
                    push_t(idx2, idx4, idx5);
                }
                push_t(idx3, idx4, idx5);
            }

            idx1 = idx3;
            idx2 = idx4;
        }
    }

    if closing {
        if post_process_close {
            // Ideally, those last two vertices could be avoided by reusing the first two. I'm
            // not sure the additional CPU cycles are worth the memory savings...
            vertices[first_idx1] = vertices[idx1];
            vertices[first_idx2] = vertices[idx2];
        }
    } else {
        // finish off the line
        let idx3 = push_v(p2 + w * (v1 + n1));
        let idx4 = push_v(p2 + w * (v1 - n1));
        push_t(idx1, idx2, idx3);
        push_t(idx2, idx3, idx4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vsvg_core::flattened_layer::FlattenedLayer;
    use vsvg_core::Document;

    fn triangulate_prealloc_pessimistic(layer: &FlattenedLayer) {
        let pts_count = layer
            .paths
            .iter()
            .map(|path| path.data.len())
            .sum::<usize>();

        let mut v = Vec::with_capacity((pts_count as f64 * 2.5) as usize);
        let mut t = Vec::with_capacity((pts_count as f64 * 2.5) as usize);
        for path in &layer.paths {
            build_fat_line(&path.data, 1.0, &mut v, &mut t);
        }
    }

    #[test]
    fn test_bar_nodef() {
        const N: usize = 1000;

        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("fixtures");
        path.push("bar_nodef.svg");
        let doc = Document::from_svg(path).unwrap().flatten(0.1);
        let layer = doc.layers.get(&0).unwrap();

        for _ in 0..N {
            triangulate_prealloc_pessimistic(layer);
        }
    }
}
