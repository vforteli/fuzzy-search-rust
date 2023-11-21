#[cfg(test)]
mod tests_extra {
    use crate::{
        fuzzy_search_levenshtein::FuzzySearchLevenshtein, fuzzy_search_options::FuzzySearchOptions,
    };

    #[test]
    fn test_all_results() {
        run_find_levenshtein_all(
            "pattern",
            "---------------------pattttern",
            21,
            "pattttern",
            2,
            3,
        );
    }

    fn run_find_levenshtein_all(
        pattern: &str,
        text: &str,
        expected_start: usize,
        expected_match: &str,
        expected_distance: usize,
        max_distance: usize,
    ) {
        let options = FuzzySearchOptions::new(max_distance);
        let all_results = FuzzySearchLevenshtein::find(pattern, text, &options).collect::<Vec<_>>();

        println!("{:?}", all_results);

        assert_eq!(all_results.len(), 24);
    }
}
