mod dnsgen_bench;
mod model_bench;
use dnsgen_bench::dnsgen_bench;
use model_bench::insert;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("insert", |b| b.iter(|| insert(black_box(2000))));
    c.bench_function("dnsgen", |b| b.iter(|| dnsgen_bench(black_box(100))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
