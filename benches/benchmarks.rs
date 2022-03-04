use criterion::{black_box, criterion_group, criterion_main, Criterion};
mod dedup;
mod execute;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("dedup", |b| b.iter(|| dedup::dedup(black_box(5000))));
    // c.bench_function("execute", |b| b.iter(|| execute::exe(black_box(2000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
