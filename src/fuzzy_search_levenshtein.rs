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
            current_text_index: 0,
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
                        }

                        self.current_text_index += 1;
                        self.candidates.push(CandidateMatch::new(
                            self.current_text_index,
                            self.current_text_index,
                        ));

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
