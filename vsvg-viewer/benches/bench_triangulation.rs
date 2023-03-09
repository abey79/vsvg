use criterion::{criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
use vsvg_core::flattened_layer::FlattenedLayer;
use vsvg_core::Document;
use vsvg_viewer::triangulation::build_fat_line_buffers;

fn triangulate_no_prealloc(layer: &FlattenedLayer) {
    let mut v = Vec::new();
    let mut t = Vec::new();
    for path in layer.paths.iter() {
        build_fat_line_buffers(&path.data, 1.0, &mut v, &mut t);
    }
}

fn triangulate_prealloc(layer: &FlattenedLayer) {
    let pts_count = layer
        .paths
        .iter()
        .map(|path| path.data.len())
        .sum::<usize>();

    let mut v = Vec::with_capacity(pts_count * 2);
    let mut t = Vec::with_capacity(pts_count * 2);
    for path in layer.paths.iter() {
        build_fat_line_buffers(&path.data, 1.0, &mut v, &mut t);
    }
}

fn triangulate_prealloc_pessimistic(layer: &FlattenedLayer) {
    let pts_count = layer
        .paths
        .iter()
        .map(|path| path.data.len())
        .sum::<usize>();

    let mut v = Vec::with_capacity((pts_count as f64 * 2.5) as usize);
    let mut t = Vec::with_capacity((pts_count as f64 * 2.5) as usize);
    for path in layer.paths.iter() {
        build_fat_line_buffers(&path.data, 1.0, &mut v, &mut t);
    }
}

fn triangulate_rayon_pessimistic(layer: &FlattenedLayer) {
    let buffers = layer
        .paths
        .par_iter()
        .map(|path| {
            let pts_count = path.data.len();
            let mut v = Vec::with_capacity((pts_count as f64 * 2.5) as usize);
            let mut t = Vec::with_capacity((pts_count as f64 * 2.5) as usize);
            build_fat_line_buffers(&path.data, 1.0, &mut v, &mut t);
            (v, t)
        })
        .collect::<Vec<_>>();

    let counts = buffers
        .par_iter()
        .map(|(v, t)| (v.len(), t.len()))
        .reduce(|| (0, 0), |(v1, t1), (v2, t2)| (v1 + v2, t1 + t2));

    let mut v = Vec::with_capacity(counts.0);
    let mut t = Vec::with_capacity(counts.1);

    let mut start_idx = 0;
    for (v_buf, t_buf) in buffers {
        let l = v_buf.len();
        v.extend(v_buf);
        t.extend(
            t_buf
                .into_iter()
                .map(|(i1, i2, i3)| (i1 + start_idx, i2 + start_idx, i3 + start_idx)),
        );
        start_idx += l;
    }
}

pub fn bench_bar_nodef(c: &mut Criterion) {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("bar_nodef.svg");
    let doc = Document::from_svg(path).unwrap().flatten(0.1);
    let layer = doc.layers.get(&0).unwrap();

    let mut group = c.benchmark_group("triangulation");

    group.bench_function("no prealloc", |b| b.iter(|| triangulate_no_prealloc(layer)));
    group.bench_function("prealloc", |b| b.iter(|| triangulate_prealloc(layer)));
    group.bench_function("prealloc pessimistic", |b| {
        b.iter(|| triangulate_prealloc_pessimistic(layer))
    });
    group.bench_function("rayon pessimistic", |b| {
        b.iter(|| triangulate_rayon_pessimistic(layer))
    });
}

criterion_group!(benches, bench_bar_nodef);
criterion_main!(benches);
