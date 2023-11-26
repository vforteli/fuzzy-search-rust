use std::{fs::File, io::Read};

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
            let _ = FuzzySearch::find(pattern, &text.chars().collect::<Vec<_>>(), &options)
                .collect::<Vec<_>>();
        })
    });

    c.bench_function("substitutions only", |b| {
        b.iter(|| {
            let _ = FuzzySearchSubstitutionsOnly::find(pattern, text, distance).collect::<Vec<_>>();
        })
    });

    c.bench_function("cia", |b| {
        let mut file =
            File::open("benches/test_files/world192.txt").expect("huh, where did the file go?");
        let mut text = String::new();
        file.read_to_string(&mut text)
            .expect("uh, failed reading file...");

        let pattern = "conftitufional"; // well, since the text probably doesnt contain bad ocr, lets introduce some errors here instead...
        let options = FuzzySearchOptions::new(distance);

        b.iter(|| {
            let _ = FuzzySearch::find(pattern, &text.chars().collect::<Vec<_>>(), &options)
                .collect::<Vec<_>>();
        })
    });

    c.bench_function("ecoli", |b| {
        let mut file =
            File::open("benches/test_files/E.coli").expect("huh, where did the file go?");
        let mut text = String::new();
        file.read_to_string(&mut text)
            .expect("uh, failed reading file...");

        let pattern = "cccctgaccatcaaccagcggataacggtaagagaacg";
        let options = FuzzySearchOptions::new(distance);

        b.iter(|| {
            let _ = FuzzySearch::find(pattern, &text.chars().collect::<Vec<_>>(), &options)
                .collect::<Vec<_>>();
        })
    });

    c.bench_function("cia_substitutions_only", |b| {
        let mut file =
            File::open("benches/test_files/world192.txt").expect("huh, where did the file go?");
        let mut text = String::new();
        file.read_to_string(&mut text)
            .expect("uh, failed reading file...");

        let pattern = "conftitufional"; // well, since the text probably doesnt contain bad ocr, lets introduce some errors here instead...

        b.iter(|| {
            let _ =
                FuzzySearchSubstitutionsOnly::find(pattern, &text, distance).collect::<Vec<_>>();
        })
    });

    c.bench_function("ecoli_substitutions_only", |b| {
        let mut file =
            File::open("benches/test_files/E.coli").expect("huh, where did the file go?");
        let mut text = String::new();
        file.read_to_string(&mut text)
            .expect("uh, failed reading file...");

        let pattern = "cccctgaccatcaaccagcGGataacggtaagagaacg";

        b.iter(|| {
            let _ =
                FuzzySearchSubstitutionsOnly::find(pattern, &text, distance).collect::<Vec<_>>();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
