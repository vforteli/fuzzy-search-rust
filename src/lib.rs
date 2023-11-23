use fuzzy_search_levenshtein::FuzzySearchLevenshtein;
use fuzzy_search_options::FuzzySearchOptions;
use match_consolidator::MatchConsolidator;
use match_result::MatchResult;

pub mod candidate_match;
mod fuzzy_search_levenshtein;
pub mod fuzzy_search_options;
pub mod fuzzy_search_substitutions_only;
mod match_consolidator;
pub mod match_result;

pub struct FuzzySearch<'a> {
    consolidated_matches: MatchConsolidator<'a, FuzzySearchLevenshtein<'a>>,
}

impl<'a> FuzzySearch<'a> {
    pub fn find(pattern: &'a str, text: &'a str, options: &'a FuzzySearchOptions) -> Self {
        Self {
            consolidated_matches: MatchConsolidator::consolidate(
                options.max_total_distance,
                text,
                FuzzySearchLevenshtein::find(pattern, text, options),
            ),
        }
    }
}

impl<'a> Iterator for FuzzySearch<'a> {
    type Item = MatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        self.consolidated_matches.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_pattern_with_grapheme() {
        run_test("PATTERN", "👩‍👩‍👦‍👦PATTERN", 0, 7, "PATTERN", 0, 1);
    }

    #[test]
    fn test_grapheme() {
        let text = "👩‍👩‍👦‍👦";
        assert_eq!(7, text.chars().collect::<Vec<_>>().len());
    }

    #[test]
    fn test_pattern_pattern() {
        run_test("PATTERN", "PATTERN", 0, 0, "PATTERN", 0, 1);
    }

    #[test]
    fn test_def_abcddefg() {
        run_test("def", "abcddefg", 0, 4, "def", 0, 1);
    }

    #[test]
    fn test_def_abcdeffg() {
        run_test("def", "abcdeffg", 1, 3, "def", 0, 1);
    }

    #[test]
    fn test_defgh_abcdedefghi() {
        run_test("defgh", "abcdedefghi", 3, 5, "defgh", 0, 1);
    }

    #[test]
    fn test_cdefgh_abcdefghghi() {
        run_test("cdefgh", "abcdefghghi", 3, 2, "cdefgh", 0, 1);
    }

    #[test]
    fn test_bde_abcdefg() {
        run_test("bde", "abcdefg", 1, 1, "bcde", 1, 1);
    }

    #[test]
    fn test_1234567_123567() {
        run_test("1234567", "--123567--", 1, 2, "123567", 1, 1);
    }

    #[test]
    fn test_1234567_1238567() {
        run_test("1234567", "--1238567--", 1, 2, "1238567", 1, 1);
    }

    #[test]
    fn test_1234567_23567() {
        run_test("1234567", "23567-----", 2, 0, "23567", 2, 1);
    }

    #[test]
    fn test_1234567_23567_dash() {
        run_test("1234567", "--23567---", 2, 1, "-23567", 2, 1);
    }

    #[test]
    fn test_1234567_dash_23567() {
        run_test("1234567", "-----23567", 2, 4, "-23567", 2, 1);
    }

    #[test]
    fn test_pattern_patt_ern_1_10() {
        run_test(
            "PATTERN",
            "----------PATT-ERN---------",
            1,
            10,
            "PATT-ERN",
            1,
            1,
        );
    }

    #[test]
    fn test_pattern_patt_ern_2_10() {
        run_test(
            "PATTERN",
            "----------PATT-ERN---------",
            2,
            10,
            "PATT-ERN",
            1,
            1,
        );
    }

    #[test]
    fn test_pattern_patttern_1_10() {
        run_test(
            "PATTERN",
            "----------PATTTERN---------",
            1,
            10,
            "PATTTERN",
            1,
            1,
        );
    }

    #[test]
    fn test_pattern_patttern_2_10() {
        run_test(
            "PATTERN",
            "----------PATTTERN---------",
            2,
            10,
            "PATTTERN",
            1,
            1,
        );
    }

    #[test]
    fn test_pattern_patternn_0_10() {
        run_test(
            "PATTERN",
            "----------PATTERNN---------",
            0,
            10,
            "PATTERN",
            0,
            1,
        );
    }

    #[test]
    fn test_pattern_patternn_1_10() {
        run_test(
            "PATTERN",
            "----------PATTERNN---------",
            1,
            10,
            "PATTERN",
            0,
            1,
        );
    }

    #[test]
    fn test_pattern_patternn_2_10() {
        run_test(
            "PATTERN",
            "----------PATTERNN---------",
            2,
            10,
            "PATTERN",
            0,
            1,
        );
    }

    #[test]
    fn test_2_deletions_buffer_start() {
        run_test("pattern", "atern----", 2, 0, "atern", 2, 1);
    }

    #[test]
    fn test_zero_max_distance_no_match() {
        run_test("pattern", "--paxxern--", 1, 0, "", 0, 0);
    }

    #[test]
    fn test_zero_max_distance_no_match_2() {
        run_test("pattern", "paxxern", 1, 0, "", 0, 0);
    }

    #[test]
    fn test_single_deletion_buffer_start() {
        run_test("pattern", "patern----", 1, 0, "patern", 1, 1);
    }

    #[test]
    fn test_single_deletion_buffer_middle() {
        run_test("pattern", "--patern--", 1, 2, "patern", 1, 1);
    }

    #[test]
    fn test_multiple_matches_consecutive() {
        run_test("pattern", "--patternpattern--", 2, 2, "pattern", 0, 2);
        run_test("pattern", "--pattern-pattern--", 1, 2, "pattern", 0, 2);
    }

    fn run_test(
        pattern: &str,
        text: &str,
        max_distance: usize,
        expected_start: usize,
        expected_match: &str,
        expected_distance: usize,
        expected_match_count: usize,
    ) {
        let options = FuzzySearchOptions::new(max_distance);
        let results = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), expected_match_count);

        if expected_match_count > 0 {
            assert_match(
                &results[0],
                expected_start,
                expected_match,
                expected_distance,
            );
        }
    }

    fn assert_match(
        result: &MatchResult,
        expected_start: usize,
        expected_match: &str,
        expected_distance: usize,
    ) {
        assert_eq!(result.match_text, expected_match);
        assert_eq!(result.start_index, expected_start);
        assert_eq!(result.distance, expected_distance);
    }

    #[test]
    fn test_options_max_substitutions() {
        let word = "pattern";
        let text = "--patteron--";
        let options = FuzzySearchOptions::with_individual_limits(1, 0, 0);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        assert_match(&results[0], 2, "pattero", 1);
    }

    #[test]
    fn test_options_max_substitutions_0() {
        let word = "patternsandpractices";
        let text = "--patternsaxdpractices--";
        let options = FuzzySearchOptions::with_limits(1, Some(0), None, None);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_options_max_insertions() {
        let word = "pattern";
        let text = "--patteron--";
        let options = FuzzySearchOptions::with_individual_limits(0, 0, 1);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        assert_match(&results[0], 2, "patteron", 1);
    }

    #[test]
    fn test_options_max_insertions_0() {
        let word = "patternsandpractices";
        let text = "--patternsaxndpractices--";
        let options = FuzzySearchOptions::with_limits(1, None, None, Some(0));

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_options_max_deletions() {
        let word = "pattern";
        let text = "--patteron--";
        let options = FuzzySearchOptions::with_individual_limits(0, 1, 0);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        assert_match(&results[0], 2, "patter", 1);
    }

    #[test]
    fn test_options_max_deletions_0() {
        let word = "patternsandpractices";
        let text = "--patternandpractices--";
        let options = FuzzySearchOptions::with_limits(1, None, Some(0), None);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_multiple_matches_consecutive_substitutions() {
        let word = "pattern";
        let text = "--pattermpatyern--";
        let options = FuzzySearchOptions::new(2);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 2);
        assert_match(&results[0], 2, "patterm", 1);
        assert_match(&results[1], 9, "patyern", 1);
    }

    #[test]
    fn test_multiple_matches_consecutive_insertion() {
        let word = "pattern";
        let text = "--patyternpatxtern--";
        let options = FuzzySearchOptions::new(1);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 2);
        assert_match(&results[0], 2, "patytern", 1);
        assert_match(&results[1], 10, "patxtern", 1);
    }

    #[test]
    fn test_overlapping_matches() {
        let word = "pattern";
        let text = "--pattpatterntern--";
        let options = FuzzySearchOptions::new(2);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);
        assert_match(&results[0], 6, "pattern", 0);
    }

    #[test]
    fn test_multiple_matches_consecutive_deletion() {
        let word = "pattern";
        let text = "--pattrnpttern--";
        let options = FuzzySearchOptions::new(2);

        let results = FuzzySearch::find(word, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 2);
        assert_match(&results[0], 2, "pattrn", 1);
        assert_match(&results[1], 8, "pttern", 1);
    }

    #[test]
    fn test_empty_text() {
        let pattern = "PATTERN";
        let text = "";
        let options = FuzzySearchOptions::new(2);

        let results = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_empty_pattern() {
        let pattern = "";
        let text = "sometext";
        let options = FuzzySearchOptions::new(2);

        let results = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_empty_pattern_and_text() {
        let pattern = "";
        let text = "";
        let options = FuzzySearchOptions::new(2);

        let results = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_shorter_text() {
        let pattern = "PATTERN";
        let text = "PATERN";
        let expected_matches = 1;
        let options = FuzzySearchOptions::new(1);

        let results = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), expected_matches);
        assert_match(&results[0], 0, "PATERN", 1);
    }

    #[test]
    fn test_shorter_text_no_match() {
        let pattern = "PATTERN";
        let text = "PAERN";
        let expected_matches = 0;
        let options = FuzzySearchOptions::new(1);

        let results = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), expected_matches);
    }

    fn run_test_other(
        pattern: &str,
        text: &str,
        expected_start: usize,
        expected_match: &str,
        expected_distance: usize,
        max_distance: usize,
    ) {
        let options = FuzzySearchOptions::new(max_distance);
        let results = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();

        assert_eq!(results.len(), 1);

        assert_match(
            &results[0],
            expected_start,
            expected_match,
            expected_distance,
        );
    }

    #[test]
    fn test_pattern_pattern_exact_match() {
        run_test_other(
            "pattern",
            "pattern---------------------",
            0,
            "pattern",
            0,
            3,
        );
    }

    #[test]
    fn test_pattern_atern_partial_match() {
        run_test_other("pattern", "attern---------------------", 0, "attern", 1, 3);
    }

    #[test]
    fn test_pattern_ttern_partial_match() {
        run_test_other("pattern", "ttern---------------------", 0, "ttern", 2, 3);
    }

    #[test]
    fn test_pattern_tern_partial_match() {
        run_test_other("pattern", "tern---------------------", 0, "tern", 3, 3);
    }

    #[test]
    fn test_pattern_pattttern_partial_match() {
        run_test_other(
            "pattern",
            "--------pattttern-------------",
            8,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_2() {
        run_test_other(
            "pattern",
            "---------pattttern------------",
            9,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_3() {
        run_test_other(
            "pattern",
            "----------pattttern-----------",
            10,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_4() {
        run_test_other(
            "pattern",
            "-----------pattttern----------",
            11,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_5() {
        run_test_other(
            "pattern",
            "------------pattttern---------",
            12,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_6() {
        run_test_other(
            "pattern",
            "-------------pattttern--------",
            13,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_7() {
        run_test_other(
            "pattern",
            "--------------pattttern-------",
            14,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_8() {
        run_test_other(
            "pattern",
            "---------------pattttern------",
            15,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_9() {
        run_test_other(
            "pattern",
            "----------------pattttern-----",
            16,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_10() {
        run_test_other(
            "pattern",
            "-----------------pattttern----",
            17,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_11() {
        run_test_other(
            "pattern",
            "------------------pattttern---",
            18,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_12() {
        run_test_other(
            "pattern",
            "-------------------pattttern--",
            19,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_13() {
        run_test_other(
            "pattern",
            "--------------------pattttern-",
            20,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_pattttern_partial_match_14() {
        run_test_other(
            "pattern",
            "---------------------pattttern",
            21,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_pattern_patter_partial_match() {
        run_test_other("pattern", "---patter", 3, "patter", 1, 3);
    }

    #[test]
    fn test_pattern_patte_partial_match() {
        run_test_other("pattern", "---patte", 3, "patte", 2, 3);
    }

    #[test]
    fn test_pattern_patt_partial_match() {
        run_test_other("pattern", "---patt", 3, "patt", 3, 3);
    }

    #[test]
    fn test_pattern_pattttern_partial_match_15() {
        run_test_other(
            "pattern",
            "----------------------pattttern",
            22,
            "pattttern",
            2,
            3,
        );
    }

    #[test]
    fn test_case_1() {
        run_test_other("ab", "-a", 1, "a", 1, 1);
    }

    #[test]
    fn test_case_2() {
        run_test_other("ab", "b---", 0, "b", 1, 1);
    }

    #[test]
    fn test_case_3() {
        run_test_other("ab", "-axb", 1, "axb", 1, 1);
    }

    #[test]
    fn test_case_4() {
        run_test_other("ab", "axb-", 0, "axb", 1, 1);
    }

    #[test]
    fn test_case_5() {
        run_test_other("ab", "--ax", 2, "ax", 1, 1);
    }

    #[test]
    fn test_case_6() {
        run_test_other("ab", "ax--", 0, "ax", 1, 1);
    }

    #[test]
    fn test_case_7() {
        run_test_other("ab", "--ab", 2, "ab", 0, 1);
    }

    #[test]
    fn test_case_8() {
        run_test_other("ab", "ab--", 0, "ab", 0, 1);
    }

    #[test]
    fn test_case_9() {
        run_test_other("ab", "ab", 0, "ab", 0, 1);
    }

    #[test]
    fn test_case_10() {
        run_test_other("ab", "-ab", 1, "ab", 0, 1);
    }

    #[test]
    fn test_case_11() {
        run_test_other("ab", "ab-", 0, "ab", 0, 1);
    }

    #[test]
    fn test_case_12() {
        run_test_other("ab", "b", 0, "b", 1, 1);
    }

    #[test]
    fn test_case_13() {
        run_test_other("ab", "a", 0, "a", 1, 1);
    }

    #[test]
    fn test_case_14() {
        run_test_other("a", "a", 0, "a", 0, 1);
    }

    #[test]
    fn test_case_15() {
        run_test_other("ab", "axb", 0, "axb", 1, 1);
    }

    #[test]
    fn test_case_16() {
        run_test_other("abc", "a", 0, "a", 2, 2);
    }

    #[test]
    fn test_case_17() {
        run_test_other("abc", "b", 0, "b", 2, 2);
    }

    #[test]
    fn test_case_18() {
        run_test_other("abc", "c", 0, "c", 2, 2);
    }

    #[test]
    fn test_case_19() {
        run_test_other("abcd", "ax", 0, "ax", 3, 3);
    }

    #[test]
    fn test_case_20() {
        run_test_other("abcd", "bx", 0, "bx", 3, 3);
    }

    #[test]
    fn test_case_21() {
        run_test_other("abcd", "cx", 0, "cx", 3, 3);
    }

    #[test]
    fn test_case_22() {
        run_test_other("abcd", "xa", 1, "a", 3, 3);
    }

    #[test]
    fn test_case_23() {
        run_test_other("abcd", "xb", 0, "xb", 3, 3);
    }

    #[test]
    fn test_case_24() {
        run_test_other("abcd", "xc", 0, "xc", 3, 3);
    }

    #[test]
    fn test_iterator() {
        let text = "---abcc----abc---axc--";
        let pattern = "abc";

        let options = FuzzySearchOptions::new(2);
        let results = FuzzySearch::find(pattern, text, &options).collect::<Vec<_>>();

        assert_eq!(3, results.len());
        assert_match(&results[0], 3, "abc", 0);
        assert_match(&results[1], 11, "abc", 0);
        assert_match(&results[2], 17, "axc", 1);

        let first = FuzzySearch::find(pattern, text, &options).next();
        assert!(first.is_some());

        assert_match(&first.unwrap(), 3, "abc", 0);
    }
}
