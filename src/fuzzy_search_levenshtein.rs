use crate::{candidate_match::CandidateMatch, fuzzy_search_options::FuzzySearchOptions};

pub struct FuzzySearchLevenshtein<'a> {
    subsequence: &'a str,
    text: &'a str,
    text_array: Vec<char>, // todo get rid of this...
    options: &'a FuzzySearchOptions,
    candidates: Vec<CandidateMatch>,
    current_text_index: usize,
    best_found_distance: usize,
}

impl<'a> FuzzySearchLevenshtein<'a> {
    pub fn find(subsequence: &'a str, text: &'a str, options: &'a FuzzySearchOptions) -> Self {
        Self {
            options,
            subsequence,
            text,
            candidates: vec![CandidateMatch::new(0, 0)],
            text_array: text.chars().collect(),
            current_text_index: if subsequence.len() == 0 {
                // this is basically here to eagerly terminate stuff if subsequence is an empty string without checking in the next function
                text.len() + 1
            } else {
                0
            },
            best_found_distance: options.max_total_distance,
        }
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

impl<'a> Iterator for FuzzySearchLevenshtein<'a> {
    type Item = CandidateMatch;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_text_index < self.text.len() {
            while let Some(candidate) = self.candidates.pop() {
                if candidate.subsequence_index == self.subsequence.len() {
                    if candidate.text_index <= self.text.len() {
                        if candidate.distance == 0 {
                            self.candidates.clear();
                            self.current_text_index += 1;
                            self.candidates.push(CandidateMatch::new(
                                self.current_text_index,
                                self.current_text_index,
                            ));
                        }

                        self.best_found_distance = candidate.distance;
                        return Some(candidate);
                    }

                    if candidate.distance == 0 {
                        self.candidates.clear();
                    }
                } else {
                    Self::handle_candidate(
                        &mut self.candidates,
                        &candidate,
                        &self.text_array,
                        self.subsequence,
                        self.best_found_distance,
                        self.options,
                        self.text.len(),
                    );
                }
            }

            self.current_text_index += 1;
            self.best_found_distance = self.options.max_total_distance;
            self.candidates.push(CandidateMatch::new(
                self.current_text_index,
                self.current_text_index,
            ));
        }

        None
    }
}

#[cfg(test)]
mod fuzzy_search_levenshtein_tests {
    use crate::{
        fuzzy_search_levenshtein::FuzzySearchLevenshtein, fuzzy_search_options::FuzzySearchOptions,
    };

    #[test]
    fn test_all_results() {
        run_find_levenshtein_all(
            "pattern",
            "---------------------pattttern",
            // 21,
            // "pattttern",
            // 2,
            3,
        );
    }

    fn run_find_levenshtein_all(
        pattern: &str,
        text: &str,
        // expected_start: usize,
        // expected_match: &str,
        // expected_distance: usize,
        max_distance: usize,
    ) {
        let options = FuzzySearchOptions::new(max_distance);
        let all_results = FuzzySearchLevenshtein::find(pattern, text, &options).collect::<Vec<_>>();

        println!("{:?}", all_results);

        assert_eq!(all_results.len(), 24);
    }
}
