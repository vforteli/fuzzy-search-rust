use crate::match_result::MatchResult;

pub struct FuzzySearchSubstitutionsOnly {
    pattern_chars: Vec<char>,
    text_chars: Vec<char>,
    max_distance: usize,
    current_text_index: usize,
    last_index: usize,
}

impl FuzzySearchSubstitutionsOnly {
    pub fn find(pattern: &str, text: &str, max_distance: usize) -> Self {
        let text_chars: Vec<_> = text.chars().collect();
        let length = text_chars.len();
        let pattern_chars: Vec<_> = pattern.chars().collect();
        let last_index = text_chars
            .len()
            .checked_sub(pattern_chars.len())
            .unwrap_or(0)
            + 1;

        Self {
            pattern_chars,
            max_distance,
            text_chars,
            current_text_index: if pattern.len() == 0 || text.len() == 0 {
                // this is basically here to eagerly terminate stuff if pattern is an empty string without checking in the next function
                length + 1
            } else {
                0
            },
            last_index,
        }
    }
}

impl Iterator for FuzzySearchSubstitutionsOnly {
    type Item = MatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_text_index < self.last_index {
            let current_index = self.current_text_index;
            self.current_text_index += 1;

            let m = &self.text_chars[current_index..current_index + self.pattern_chars.len()]
                .iter()
                .zip(&self.pattern_chars)
                .try_fold(0, |a, v| {
                    let distance = match v.0 == v.1 {
                        true => a,
                        false => a + 1,
                    };

                    match distance > self.max_distance {
                        true => None,
                        false => Some(distance),
                    }
                });

            if let Some(distance) = m {
                return Some(MatchResult {
                    start_index: current_index,
                    end_index: current_index + self.pattern_chars.len(),
                    distance: *distance,
                    match_text: self.text_chars
                        [current_index..(current_index + self.pattern_chars.len())]
                        .iter()
                        .collect::<String>(),
                    deletions: 0,
                    insertions: 0,
                    substitutions: *distance,
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
    fn test_pattern_pattern_with_grapheme() {
        let pattern = "PATTERN";
        let text = "👩‍👩‍👦‍👦PATTERN";

        let matches = FuzzySearchSubstitutionsOnly::find(pattern, text, 1).collect::<Vec<_>>();
        let m = &matches[0];

        assert_eq!(pattern, m.match_text);
        assert_eq!(0, m.distance);
        assert_eq!(7, m.start_index); // todo this is a bit weird now since the index refers to the char array...
        assert_eq!(14, m.end_index); // todo same story here...
    }
    #[test]
    fn test_grapheme_with_empty_pattern() {
        let pattern = "";
        let text = "👩‍👩‍👦‍👦PATTERN";

        let matches = FuzzySearchSubstitutionsOnly::find(pattern, text, 1).collect::<Vec<_>>();
        assert_eq!(0, matches.len());
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
