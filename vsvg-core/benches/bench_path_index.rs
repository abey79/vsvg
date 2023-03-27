use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::Rng;
use std::time::Duration;
use vsvg_core::flattened_layer::FlattenedLayer;
use vsvg_core::path_index::{IndexBuilder, ReindexStrategy};
use vsvg_core::point::Point;
use vsvg_core::{test_file, Document};

pub fn bench_path_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_index");
    group.sample_size(15);

    for file in ["5k", "15k", "50k", "100k", "200k"] {
        let doc =
            Document::from_svg(test_file!(format!("{file}_random_lines.svg")), false).unwrap();
        let layer = doc.try_get(1).unwrap();

        for ratio in [
            0.01, 0.025, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.75, 1.0,
        ] {
            for flip in [false, true] {
                group.bench_function(
                    format!(
                        "{file}_ratio_{ratio}_{}",
                        if flip { "flip" } else { "noflip" }
                    ),
                    |b| {
                        b.iter_batched(
                            || layer.clone(),
                            |mut layer| {
                                layer.sort_with_builder(
                                    IndexBuilder::new()
                                        .strategy(ReindexStrategy::Ratio(ratio))
                                        .flip(flip),
                                )
                            },
                            BatchSize::SmallInput,
                        )
                    },
                );
            }
        }
    }
}

pub fn bench_huge_index(c: &mut Criterion) {
    let mut layer = FlattenedLayer::new();
    let mut rng = rand::thread_rng();
    const NUM: usize = 1_000_000;
    //let num: f64 = task_rng().gen_range(-40.0, 1.3e4);
    layer.paths = (0..NUM)
        .map(|_i| {
            vec![
                Point::new(rng.gen_range(0.0..1.), rng.gen_range(0.0..1.)),
                Point::new(rng.gen_range(0.0..1.), rng.gen_range(0.0..1.)),
            ]
            .into()
        })
        .collect();

    let mut group = c.benchmark_group("huge_index");
    group
        .measurement_time(Duration::from_secs(120))
        .sample_size(10);

    for ratio in [0.20, 0.3, 0.4, 0.5, 0.6, 0.75, 1.0] {
        group.bench_function(format!("{}M_ratio_{ratio}_flip", NUM / 1_000_000), |b| {
            b.iter_batched(
                || layer.clone(),
                |mut layer| {
                    layer.sort_with_builder(
                        IndexBuilder::new()
                            .strategy(ReindexStrategy::Ratio(ratio))
                            .flip(true),
                    )
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, bench_path_index, bench_huge_index);
criterion_main!(benches);
