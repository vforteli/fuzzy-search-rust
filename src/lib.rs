use candidate_match::CandidateMatch;
use fuzzy_search_options::FuzzySearchOptions;
use match_result::MatchResult;

pub mod candidate_match;
pub mod fuzzy_search_options;
pub mod match_result;
pub mod match_result_value;

pub struct FuzzySearch;

impl FuzzySearch {
    pub fn find_levenshtein(
        subsequence: &str,
        text: &str,
        options: &FuzzySearchOptions,
    ) -> Vec<MatchResult> {
        Self::consolidate_matches(
            text,
            FuzzySearch::find_levenshtein_all(subsequence, text, options),
            options.max_total_distance,
        )
    }

    pub fn find_levenshtein_all(
        subsequence: &str,
        text: &str,
        options: &FuzzySearchOptions,
    ) -> Vec<CandidateMatch> {
        if subsequence.is_empty() {
            return Vec::new();
        }

        // todo this should be removed and the whole function be converted to an iterator
        let mut results = Vec::new();

        let mut candidates = Vec::new();
        let text_array: Vec<char> = text.chars().collect();

        for current_index in 0..text.len() {
            candidates.push(CandidateMatch::new(current_index, current_index));

            let mut best_found_distance = options.max_total_distance;

            while let Some(candidate) = candidates.pop() {
                if candidate.subsequence_index == subsequence.len() {
                    if candidate.text_index <= text.len() {
                        best_found_distance = candidate.distance;
                        results.push(candidate);
                    }

                    if candidate.distance == 0 {
                        candidates.clear();
                        break;
                    }

                    continue;
                }

                FuzzySearch::handle_candidate(
                    &mut candidates,
                    &candidate,
                    &text_array,
                    subsequence,
                    best_found_distance,
                    options,
                    text.len(),
                );
            }
        }

        results
    }

    #[inline(always)]
    fn handle_candidate(
        candidates: &mut Vec<CandidateMatch>,
        candidate: &CandidateMatch,
        text: &[char],
        subsequence: &str,
        best_found_distance: usize,
        options: &FuzzySearchOptions,
        text_length: usize,
    ) {
        if candidate.text_index < text_length
            && text[candidate.text_index]
                == subsequence
                    .chars()
                    .nth(candidate.subsequence_index)
                    .unwrap()
        {
            if candidate.distance < best_found_distance
                && options.can_insert(candidate.distance, candidate.insertions)
            {
                // jump over one character in text
                candidates.push(CandidateMatch {
                    insertions: candidate.insertions + 1,
                    distance: candidate.distance + 1,
                    subsequence_index: candidate.subsequence_index + 1,
                    text_index: candidate.text_index + 2,
                    ..*candidate
                });
            }

            // match
            candidates.push(CandidateMatch {
                text_index: candidate.text_index + 1,
                subsequence_index: candidate.subsequence_index + 1,
                ..*candidate
            });
        } else if candidate.distance < best_found_distance {
            if options.can_delete(candidate.distance, candidate.deletions) {
                // jump over one character in subsequence
                candidates.push(CandidateMatch {
                    deletions: candidate.deletions + 1,
                    distance: candidate.distance + 1,
                    subsequence_index: candidate.subsequence_index + 1,
                    ..*candidate
                });
            }

            if options.can_substitute(candidate.distance, candidate.substitutions) {
                // substitute one character
                candidates.push(CandidateMatch {
                    substitutions: candidate.substitutions + 1,
                    distance: candidate.distance + 1,
                    text_index: candidate.text_index + 1,
                    subsequence_index: candidate.subsequence_index + 1,
                    ..*candidate
                });
            }
        }
    }

    /// Group matches and return best.
    /// Currently assumes the matches are in the same order they are found...
    fn consolidate_matches(
        text: &str,
        matches: Vec<CandidateMatch>,
        max_distance: usize,
    ) -> Vec<MatchResult> {
        let mut matches_iter = matches.into_iter();
        let mut group = Vec::new();

        let mut results = Vec::new(); // todo iterator...

        if let Some(first_match) = matches_iter.next() {
            group.push(first_match);

            let mut match_start_index = first_match.start_index;

            while let Some(next_match) = matches_iter.next() {
                if next_match.start_index > (match_start_index + max_distance as usize) {
                    if !group.is_empty() {
                        results.push(Self::get_best_match_from_group(&group, text));
                        group.clear();
                    }
                }

                group.push(next_match);
                match_start_index = next_match.start_index;
            }
        }

        if !group.is_empty() {
            results.push(Self::get_best_match_from_group(&group, text));
        }

        results
    }

    #[inline(always)]
    fn get_best_match_from_group(group: &Vec<CandidateMatch>, text: &str) -> MatchResult {
        let mut best_match = group.first().unwrap();

        // todo figure out if we can get rid of the checked_sub by ensuring it never is negative...
        for match_item in group.iter().skip(1) {
            if match_item.distance < best_match.distance
                || (match_item.distance == best_match.distance
                    && (match_item
                        .start_index
                        .checked_sub(match_item.text_index)
                        .unwrap_or(0))
                        > (best_match
                            .start_index
                            .checked_sub(best_match.text_index)
                            .unwrap_or(0)))
            {
                best_match = match_item;
            }
        }

        MatchResult {
            start_index: best_match.start_index,
            end_index: best_match.text_index,
            distance: best_match.distance,
            match_text: text[best_match.start_index..best_match.text_index].to_string(),
            deletions: best_match.deletions,
            insertions: best_match.insertions,
            substitutions: best_match.substitutions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_max_distance_no_match() {
        let word = "pattern";
        let text = "--paxxern--";

        let options = FuzzySearchOptions::new(1);
        let results = FuzzySearch::find_levenshtein(word, text, &options);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_pattern_pattern() {
        run_test("PATTERN", "PATTERN", 0, 0, "PATTERN", 0);
    }

    #[test]
    fn test_def_abcddefg() {
        run_test("def", "abcddefg", 0, 4, "def", 0);
    }

    #[test]
    fn test_def_abcdeffg() {
        run_test("def", "abcdeffg", 1, 3, "def", 0);
    }

    #[test]
    fn test_defgh_abcdedefghi() {
        run_test("defgh", "abcdedefghi", 3, 5, "defgh", 0);
    }

    #[test]
    fn test_cdefgh_abcdefghghi() {
        run_test("cdefgh", "abcdefghghi", 3, 2, "cdefgh", 0);
    }

    #[test]
    fn test_bde_abcdefg() {
        run_test("bde", "abcdefg", 1, 1, "bcde", 1);
    }

    #[test]
    fn test_1234567_123567() {
        run_test("1234567", "--123567--", 1, 2, "123567", 1);
    }

    #[test]
    fn test_1234567_1238567() {
        run_test("1234567", "--1238567--", 1, 2, "1238567", 1);
    }

    #[test]
    fn test_1234567_23567() {
        run_test("1234567", "23567-----", 2, 0, "23567", 2);
    }

    #[test]
    fn test_1234567_23567_dash() {
        run_test("1234567", "--23567---", 2, 1, "-23567", 2);
    }

    #[test]
    fn test_1234567_dash_23567() {
        run_test("1234567", "-----23567", 2, 4, "-23567", 2);
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
        );
    }

    fn run_test(
        pattern: &str,
        text: &str,
        max_distance: usize,
        expected_start: usize,
        expected_match: &str,
        expected_distance: usize,
    ) {
        let options = FuzzySearchOptions::new(max_distance);
        let results = FuzzySearch::find_levenshtein(pattern, text, &options);

        assert_eq!(results.len(), 1);
        assert_match(
            &results[0],
            expected_start,
            expected_match,
            expected_distance,
        );
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
