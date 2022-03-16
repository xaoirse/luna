use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(_c: &mut Criterion) {
    // c.bench_function("execute", |b| b.iter(|| execute::exe(black_box(2000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
