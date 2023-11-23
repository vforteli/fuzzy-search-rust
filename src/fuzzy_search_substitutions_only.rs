use crate::match_result::MatchResult;

pub struct FuzzySearchSubstitutionsOnly<'a> {
    pattern_chars: Vec<char>, // todo get rid of this...
    text: &'a str,
    text_chars: Vec<char>, // todo get rid of this...
    max_distance: usize,
    current_text_index: usize,
}

impl<'a> FuzzySearchSubstitutionsOnly<'a> {
    pub fn find(pattern: &'a str, text: &'a str, max_distance: usize) -> Self {
        Self {
            pattern_chars: pattern.chars().collect(),
            text,
            max_distance,
            text_chars: text.chars().collect(),
            current_text_index: if pattern.len() == 0 || text.len() == 0 {
                // this is basically here to eagerly terminate stuff if pattern is an empty string without checking in the next function
                text.len() + 1
            } else {
                0
            },
        }
    }
}

impl<'a> Iterator for FuzzySearchSubstitutionsOnly<'a> {
    type Item = MatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_text_index
            < self
                .text_chars
                .len()
                .checked_sub(self.pattern_chars.len())
                .unwrap_or(0)
                + 1
        {
            let current_index = self.current_text_index;
            self.current_text_index += 1;

            let mut needle_position = current_index;
            let mut candidate_distance = 0;

            for pattern_index in 0..self.pattern_chars.len() {
                if self.text_chars[needle_position] != self.pattern_chars[pattern_index] {
                    candidate_distance += 1;
                    if candidate_distance > self.max_distance {
                        break;
                    }
                }

                needle_position += 1;
            }

            if candidate_distance <= self.max_distance {
                return Some(MatchResult {
                    start_index: current_index,
                    end_index: current_index + self.pattern_chars.len(),
                    distance: candidate_distance,
                    match_text: self.text
                        [current_index..(current_index + self.pattern_chars.len())]
                        .to_string(), // this will explode with graphemes.. probably
                    deletions: 0,
                    insertions: 0,
                    substitutions: candidate_distance,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod fuzzy_search_substitution_only_tests {
    use super::*;

    fn assert_match(start_index: usize, end_index: usize, text: &str, m: &MatchResult) {
        assert_eq!(start_index, m.start_index);
        assert_eq!(end_index, m.end_index);
        assert_eq!(text[start_index..end_index], m.match_text);
    }

    #[test]
    fn multiple_matches() {
        let pattern = "foo";
        let text = "foo--fo----f--f-oo";

        let matches = FuzzySearchSubstitutionsOnly::find(pattern, text, 1).collect::<Vec<_>>();

        assert_eq!(4, matches.len());

        assert_match(0, 3, text, &matches[0]);
        assert_match(5, 8, text, &matches[1]);
        assert_match(14, 17, text, &matches[2]);
        assert_match(15, 18, text, &matches[3]);
    }

    #[test]
    fn tgcactgtagggataacaat() {
        let pattern = "TGCACTGTAGGGATAACAAT";
        let text = "GACTAGCACTGTAGGGATAACAATTTCACACAGGTGGACAATTACATTGAAAATCACAGATTGGTCACACACACATTGGACATACATAGAAACACACACACATACATTAGATACGAACATAGAAACACACATTAGACGCGTACATAGACACAAACACATTGACAGGCAGTTCAGATGATGACGCCCGACTGATACTCGCGTAGTCGTGGGAGGCAAGGCACACAGGGGATAGG";

        let matches = FuzzySearchSubstitutionsOnly::find(pattern, text, 2).collect::<Vec<_>>();

        assert_eq!(1, matches.len());
        assert_eq!(4, matches[0].start_index);
        assert_eq!(24, matches[0].end_index);
        assert_eq!(text[4..24], matches[0].match_text);
    }

    #[test]
    fn gggttlttss() {
        let pattern = "GGGTTLTTSS";
        let text = "XXXXXXXXXXXXXXXXXXXGGGTTVTTSSAAAAAAAAAAAAAGGGTTLTTSSAAAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBBBBBBBBBBBGGGTTLTTSS";

        {
            let matches_0_distance =
                FuzzySearchSubstitutionsOnly::find(pattern, text, 0).collect::<Vec<_>>();

            assert_eq!(2, matches_0_distance.len());
            assert_match(42, 52, text, &matches_0_distance[0]);
            assert_match(99, 109, text, &matches_0_distance[1]);
        }
        {
            let matches_1_distance =
                FuzzySearchSubstitutionsOnly::find(pattern, text, 1).collect::<Vec<_>>();

            assert_eq!(3, matches_1_distance.len());
            assert_match(19, 29, text, &matches_1_distance[0]);
            assert_match(42, 52, text, &matches_1_distance[1]);
            assert_match(99, 109, text, &matches_1_distance[2]);
        }
        {
            let matches_2_distance =
                FuzzySearchSubstitutionsOnly::find(pattern, text, 2).collect::<Vec<_>>();

            assert_eq!(3, matches_2_distance.len());
            assert_match(19, 29, text, &matches_2_distance[0]);
            assert_match(42, 52, text, &matches_2_distance[1]);
            assert_match(99, 109, text, &matches_2_distance[2]);
        }
    }

    #[test]
    fn no_match() {
        let pattern = "foo";
        let text = "f-------f----o---f---o-o--";

        assert_eq!(
            0,
            FuzzySearchSubstitutionsOnly::find(pattern, text, 1)
                .collect::<Vec<_>>()
                .len()
        );
    }

    #[test]
    fn perfect_match() {
        let pattern = "foo";
        let text = "foo";

        let matches = FuzzySearchSubstitutionsOnly::find(pattern, text, 1).collect::<Vec<_>>();

        assert_eq!(1, matches.len());
        assert_eq!(0, matches[0].start_index);
        assert_eq!(3, matches[0].end_index);
        assert_eq!(text, matches[0].match_text);
    }

    #[test]
    fn empty_pattern() {
        let pattern = "";
        let text = "foo";

        assert_eq!(
            0,
            FuzzySearchSubstitutionsOnly::find(pattern, text, 2)
                .collect::<Vec<_>>()
                .len()
        );
    }

    #[test]
    fn empty_text() {
        let pattern = "foo";
        let text = "";

        assert_eq!(
            0,
            FuzzySearchSubstitutionsOnly::find(pattern, text, 2)
                .collect::<Vec<_>>()
                .len()
        );
    }

    #[test]
    fn empty_pattern_and_text() {
        let pattern = "";
        let text = "";

        assert_eq!(
            0,
            FuzzySearchSubstitutionsOnly::find(pattern, text, 2)
                .collect::<Vec<_>>()
                .len()
        );
    }
}
