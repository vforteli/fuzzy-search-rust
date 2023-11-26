# fuzzysearchrs

Fuzzy search library written in rust. Basically a port of [FuzzySearch.Net](https://github.com/vforteli/FuzzySearch.Net) which in turn is inspired by [fuzzysearch](https://github.com/taleinat/fuzzysearch).

This is an online version of fuzzy search and does not make use of indices and therefore is not the fastest fuzzy search around. It was mainly created for handling OCRed documents with errors.

If insertions and deletions are not required, using substitutions only is normally at least an order of magnitude faster.

## Usage


Searching for strings in strings
``` rust
let pattern = "sometext";
let text = "here is someteext for you";

// Search with maximum distance 3, insertions, substitutions, deletions allowed
let options = FuzzySearchOptions::new(3);
let results = FuzzySearch::find(pattern, &text.chars().collect::<Vec<_>>(), &options)
        .collect::<Vec<_>>();

// Search using only substitutions and maximum distance 3
let results =  FuzzySearchSubstitutionsOnly::find(pattern, text, 3);

// Search using with more specific options, for example allowing more substitutions than insertions and deletions
let options = FuzzySearchOptions::with_individual_limits(3, 1, 1);
let results = FuzzySearch::find(pattern, &text.chars().collect::<Vec<_>>(), &options)
        .collect::<Vec<_>>();

// Check for any matches using Iterator. Using next on the Iterator is more efficient since enumeration will stop after first match.
// This will not necessarily yield the best match though.
let first = FuzzySearch::find(pattern, &text.chars().collect::<Vec<_>>(), &options).next();
    assert!(first.is_some());
```