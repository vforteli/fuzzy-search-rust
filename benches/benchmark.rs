use criterion::{criterion_group, criterion_main, Criterion};
use fuzzysearchrs::{
    fuzzy_search_options::FuzzySearchOptions,
    fuzzy_search_substitutions_only::FuzzySearchSubstitutionsOnly,
    fuzzy_search_substitutions_only_simd::FuzzySearchSubstitutionsOnlySimdTest, FuzzySearch,
};
use test_data::{get_cia_text, get_ecoli_text, MEDIUM_PATTERN, MEDIUM_TEXT};

pub mod test_data;

fn bench_standard(c: &mut Criterion) {
    let distance = 3;
    let options = FuzzySearchOptions::new(distance);

    c.bench_function("some bench", |b| {
        b.iter(|| {
            let _ = FuzzySearch::find(
                MEDIUM_PATTERN,
                &MEDIUM_TEXT.chars().collect::<Vec<_>>(),
                &options,
            )
            .collect::<Vec<_>>();
        })
    });

    c.bench_function("substitutions only", |b| {
        b.iter(|| {
            let _ = FuzzySearchSubstitutionsOnly::find(MEDIUM_PATTERN, MEDIUM_TEXT, distance)
                .collect::<Vec<_>>();
        })
    });

    c.bench_function("substitutions only simd", |b| {
        b.iter(|| {
            let _ =
                FuzzySearchSubstitutionsOnlySimdTest::find(MEDIUM_PATTERN, MEDIUM_TEXT, distance)
                    .collect::<Vec<_>>();
        })
    });

    c.bench_function("cia", |b| {
        let text = get_cia_text();
        let pattern = "conftitufional"; // well, since the text probably doesnt contain bad ocr, lets introduce some errors here instead...
        let options = FuzzySearchOptions::new(distance);

        b.iter(|| {
            let _ = FuzzySearch::find(pattern, &text.chars().collect::<Vec<_>>(), &options)
                .collect::<Vec<_>>();
        })
    });

    c.bench_function("ecoli", |b| {
        let text = get_ecoli_text();
        let pattern = "cccctgaccatcaaccagcggataacggtaagagaacg";
        let options = FuzzySearchOptions::new(distance);

        b.iter(|| {
            let _ = FuzzySearch::find(pattern, &text.chars().collect::<Vec<_>>(), &options)
                .collect::<Vec<_>>();
        })
    });

    c.bench_function("cia_substitutions_only", |b| {
        let text = get_cia_text();
        let pattern = "conftitufional"; // well, since the text probably doesnt contain bad ocr, lets introduce some errors here instead...

        b.iter(|| {
            let _ =
                FuzzySearchSubstitutionsOnly::find(pattern, &text, distance).collect::<Vec<_>>();
        })
    });

    c.bench_function("ecoli_substitutions_only", |b| {
        let text = get_ecoli_text();
        let pattern = "cccctgaccatcaaccagcGGataacggtaagagaacg";

        b.iter(|| {
            let _ =
                FuzzySearchSubstitutionsOnly::find(pattern, &text, distance).collect::<Vec<_>>();
        })
    });
}

fn bench_simd_substitutions_only_cia(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_substitutions_only_cia");

    let text = get_cia_text();
    let pattern = "conftitufional"; // well, since the text probably doesnt contain bad ocr, lets introduce some errors here instead...
    let max_distance = 3;

    group.bench_function("simd_nope", |b| {
        b.iter(|| {
            let _ = FuzzySearchSubstitutionsOnly::find(pattern, &text, max_distance)
                .collect::<Vec<_>>();
        });
    });

    group.bench_function("simd_yep", |b| {
        b.iter(|| {
            let _ = FuzzySearchSubstitutionsOnlySimdTest::find(pattern, &text, max_distance)
                .collect::<Vec<_>>();
        });
    });

    group.finish();
}

fn bench_simd_substitutions_only_ecoli(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_substitutions_only_ecoli");

    let text = get_ecoli_text();
    let pattern = "cccctgaccatcaaccagcGGataacggtaagagaacg";
    let max_distance = 3;

    group.bench_function("simd_nope", |b| {
        b.iter(|| {
            let _ = FuzzySearchSubstitutionsOnly::find(pattern, &text, max_distance)
                .collect::<Vec<_>>();
        });
    });

    group.bench_function("simd_yep", |b| {
        b.iter(|| {
            let _ = FuzzySearchSubstitutionsOnlySimdTest::find(pattern, &text, max_distance)
                .collect::<Vec<_>>();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_standard,
    bench_simd_substitutions_only_cia,
    bench_simd_substitutions_only_ecoli
);
criterion_main!(benches);
