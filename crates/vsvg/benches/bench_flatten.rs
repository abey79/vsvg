use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::{Rng, SeedableRng};
use vsvg::LayerTrait;

pub fn bench_flatten(c: &mut Criterion) {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);

    let catmull_rom = vsvg::CatmullRom::from_points(
        (0..1500).map(|_| vsvg::Point::new(rng.gen_range(0.0..1000.0), rng.gen_range(0.0..1000.0))),
    );

    let path = vsvg::Path::from(catmull_rom.clone());

    let mut group = c.benchmark_group("flatten");
    group.bench_function("catmull_1500_bezpath_el", |b| {
        b.iter_batched(
            || path.clone(),
            |path| path.flatten(0.01),
            BatchSize::SmallInput,
        )
    });

    let mut layer = vsvg::Layer::new();
    for pt in catmull_rom.points() {
        layer.push_path(kurbo::Circle::new(*pt, 1.0));
    }

    group.bench_function("1500_circles", |b| {
        b.iter_batched(
            || layer.clone(),
            |layer| layer.flatten(0.01),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_flatten);
criterion_main!(benches);
