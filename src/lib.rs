use fuzzy_search_levenshtein::FuzzySearchLevenshtein;
use fuzzy_search_options::FuzzySearchOptions;
use match_consolidator::MatchConsolidator;
use match_result::MatchResult;

pub mod candidate_match;
mod fuzzy_search_levenshtein;
pub mod fuzzy_search_options;
mod match_consolidator;
pub mod match_result;
pub mod match_result_value;

pub struct FuzzySearch;

impl FuzzySearch {
    pub fn find(subsequence: &str, text: &str, options: &FuzzySearchOptions) -> Vec<MatchResult> {
        let mut matches = FuzzySearchLevenshtein::find(subsequence, text, options);

        MatchConsolidator::consolidate(options.max_total_distance, text, &mut matches).collect()

        // Self::consolidate_matches(text, &mut matches, options.max_total_distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let results = FuzzySearch::find(pattern, text, &options);

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
        assert_eq!(result.start_index, expected_start);
        assert_eq!(result.match_text, expected_match);
        assert_eq!(result.distance, expected_distance);
    }
}

// todo rest...
