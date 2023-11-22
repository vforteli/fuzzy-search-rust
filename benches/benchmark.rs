use criterion::{criterion_group, criterion_main, Criterion};
use fuzzysearchrs::{
    fuzzy_search_options::FuzzySearchOptions,
    fuzzy_search_substitutions_only::FuzzySearchSubstitutionsOnly, FuzzySearch,
};

fn criterion_benchmark(c: &mut Criterion) {
    let distance = 3;
    let pattern = "fooo--foo-----fo";
    let text = "foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--";
    let options = FuzzySearchOptions::new(distance);

    c.bench_function("some bench", |b| {
        b.iter(|| {
            let _ = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();
        })
    });

    c.bench_function("substitutions only", |b| {
        b.iter(|| {
            let _ = FuzzySearchSubstitutionsOnly::find(pattern, text, distance).collect::<Vec<_>>();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
