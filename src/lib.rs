use candidate_match::CandidateMatch;
use fuzzy_search_options::FuzzySearchOptions;
use match_result::MatchResult;

mod candidate_match;
mod fuzzy_search_options;
mod match_result;
mod match_result_value;

pub struct FuzzySearch;

impl FuzzySearch {
    pub fn find_levenshtein(
        subsequence: &str,
        text: &str,
        options: &FuzzySearchOptions,
    ) -> Vec<MatchResult> {
        let results = FuzzySearch::find_levenshtein_all(subsequence, text, options);

        // todo so this should actually be run through the match consolidation logic...
        // todo should return an iterator
        // todo lots of stuff...
        results
            .iter()
            .map(|v| MatchResult {
                deletions: v.deletions,
                start_index: v.start_index,
                end_index: v.text_index,
                distance: v.distance,
                match_text: text[v.start_index..v.text_index].to_string(),
                substitutions: v.substitutions,
                insertions: v.insertions,
            })
            .collect()
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
