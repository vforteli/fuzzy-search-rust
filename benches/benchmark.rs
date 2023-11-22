use criterion::{criterion_group, criterion_main, Criterion};
use fuzzysearchrs::{fuzzy_search_options::FuzzySearchOptions, FuzzySearch};

fn criterion_benchmark(c: &mut Criterion) {
    let pattern = "fooo--foo-----fo";
    let text = "foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--";
    let options = FuzzySearchOptions::new(1);

    c.bench_function("some bench", |b| {
        b.iter(|| {
            let _ = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
