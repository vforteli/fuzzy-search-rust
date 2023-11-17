use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("some bench", |b| {
        b.iter(|| panic!("nope... not implemented"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
