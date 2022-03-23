mod dnsgen_bench;
use dnsgen_bench::dnsgen;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("dnsgen", |b| b.iter(|| dnsgen(black_box(2000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
