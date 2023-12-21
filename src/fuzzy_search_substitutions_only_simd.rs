use std::{
    collections::VecDeque,
    mem,
    simd::{prelude::SimdPartialEq, Simd},
};

use crate::match_result::MatchResult;

pub struct FuzzySearchSubstitutionsOnlySimdTest {
    pattern_u32s: Vec<u32>,
    text_u32s: Vec<u32>,
    max_distance: usize,
    current_text_index: usize,
    last_index: usize,
    last_index_simd: usize,
    match_buffer: VecDeque<MatchResult>, // since we are using simd, calling next may produce multiple matches.. they are buffered here and returned one by one
}

const LANES: usize = 64;

impl FuzzySearchSubstitutionsOnlySimdTest {
    pub fn find(pattern: &str, text: &str, max_distance: usize) -> Self {
        unsafe {
            let text_u32s: Vec<u32> = mem::transmute(text.chars().collect::<Vec<_>>());
            let pattern_u32s: Vec<u32> = mem::transmute(pattern.chars().collect::<Vec<_>>());
            let length = text_u32s.len();
            let last_index = text_u32s.len().checked_sub(pattern_u32s.len()).unwrap_or(0) + 1;

            Self {
                text_u32s,
                pattern_u32s,
                max_distance,
                current_text_index: if pattern.len() == 0 || text.len() == 0 {
                    // this is basically here to eagerly terminate stuff if pattern is an empty string without checking in the next function
                    length + 1
                } else {
                    0
                },
                last_index,
                last_index_simd: last_index.checked_sub(LANES).unwrap_or(0),
                match_buffer: VecDeque::new(),
            }
        }
    }
}

impl Iterator for FuzzySearchSubstitutionsOnlySimdTest {
    type Item = MatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(m) = self.match_buffer.pop_back() {
            return Some(m);
        }

        let remaining_chars = self
            .last_index
            .checked_sub(self.current_text_index)
            .unwrap_or(0);

        if remaining_chars >= LANES {
            while self.current_text_index < self.last_index_simd {
                let current_index = self.current_text_index;
                self.current_text_index += LANES;

                assert!(self.text_u32s.len() >= current_index + self.pattern_u32s.len()); // so this actually does work :O perf gain around 5%, gets rid of the bounds check in the from_slice call

                // using a mutable distance_vector here intead of fold yields slighly better performance in benchmarks
                let mut distance_vector: Simd<i32, LANES> =
                    Simd::splat(self.pattern_u32s.len() as i32);
                self.pattern_u32s.iter().enumerate().for_each(|(i, p)| {
                    let pattern_vector: Simd<u32, LANES> = Simd::splat(*p);
                    let text_vector: Simd<u32, LANES> =
                        Simd::from_slice(&self.text_u32s[current_index + i..]);

                    distance_vector += SimdPartialEq::simd_eq(text_vector, pattern_vector).to_int();
                });

                distance_vector
                    .as_array()
                    .iter()
                    .enumerate()
                    .filter(|f| *f.1 <= self.max_distance as i32)
                    .for_each(|(i, v)| {
                        self.match_buffer.push_front(MatchResult {
                            start_index: current_index + i,
                            end_index: current_index + i + self.pattern_u32s.len(),
                            distance: *v as usize,
                            match_text: {
                                unsafe {
                                    let transmuted: &[char] = mem::transmute(
                                        &self.text_u32s[current_index + i
                                            ..(current_index + i + self.pattern_u32s.len())],
                                    );

                                    transmuted.iter().collect::<String>()
                                }
                            },
                            deletions: 0,
                            insertions: 0,
                            substitutions: *v as usize,
                        });
                    });

                if let Some(m) = self.match_buffer.pop_back() {
                    return Some(m);
                }
            }
        } else {
            // this is here to handle the remaining chars which dont fit into lanes width
            while self.current_text_index < self.last_index {
                let current_index = self.current_text_index;
                self.current_text_index += 1;

                let m = &self.text_u32s[current_index..current_index + self.pattern_u32s.len()]
                    .iter()
                    .zip(&self.pattern_u32s)
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
                        end_index: current_index + self.pattern_u32s.len(),
                        distance: *distance,
                        match_text: {
                            unsafe {
                                let transmuted: &[char] = mem::transmute(
                                    &self.text_u32s
                                        [current_index..(current_index + self.pattern_u32s.len())],
                                );

                                transmuted.iter().collect::<String>()
                            }
                        },
                        deletions: 0,
                        insertions: 0,
                        substitutions: *distance,
                    });
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod fuzzy_search_substitution_only_simd_tests {
    use super::*;

    #[test]
    fn test_something_simd() {
        let text = "--------patxexn----------pattern---------------paxxern---pattern";
        let pattern = "pattern";
        let max_distance = 2;

        let matches = FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, max_distance)
            .collect::<Vec<_>>();

        assert_eq!(4, matches.len());

        assert_match(8, 15, text, &matches[0]);
        assert_match(25, 32, text, &matches[1]);
        assert_match(47, 54, text, &matches[2]);
        assert_match(57, 64, text, &matches[3]);
    }

    fn assert_match(start_index: usize, end_index: usize, text: &str, m: &MatchResult) {
        assert_eq!(start_index, m.start_index);
        assert_eq!(end_index, m.end_index);
        assert_eq!(text[start_index..end_index], m.match_text);
    }

    #[test]
    fn test_pattern_pattern_with_grapheme() {
        let pattern = "PATTERN";
        let text = "👩‍👩‍👦‍👦PATTERN-------------------------------------------------------------------------------------------------------------------------------------------------------";

        let matches =
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 1).collect::<Vec<_>>();
        let m = &matches[0];

        assert_eq!(pattern, m.match_text);
        assert_eq!(0, m.distance);
        assert_eq!(7, m.start_index); // todo this is a bit weird now since the index refers to the char array...
        assert_eq!(14, m.end_index); // todo same story here...
    }
    #[test]
    fn test_grapheme_with_empty_pattern() {
        let pattern = "";
        let text = "👩‍👩‍👦‍👦PATTERN-------------------------------------------------------------------------------------------------------------------------------------------------------";

        let matches =
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 1).collect::<Vec<_>>();
        assert_eq!(0, matches.len());
    }

    #[test]
    fn multiple_matches_short() {
        let pattern = "fo";
        let text = "f1f2f3f4-------------------------------------------------------------------------------------------------------------------------------------------------------";

        let matches =
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 1).collect::<Vec<_>>();

        assert_eq!(4, matches.len());

        assert_match(0, 2, text, &matches[0]);
        assert_match(2, 4, text, &matches[1]);
        assert_match(4, 6, text, &matches[2]);
        assert_match(6, 8, text, &matches[3]);
    }

    #[test]
    fn multiple_matches() {
        let pattern = "foo";
        let text = "foo--fo----f--f-oo-------------------------------------------------------------------------------------------------------------------------------------------------------";

        let matches =
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 1).collect::<Vec<_>>();

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

        let matches =
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 2).collect::<Vec<_>>();

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
                FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 0).collect::<Vec<_>>();

            assert_eq!(2, matches_0_distance.len());
            assert_match(42, 52, text, &matches_0_distance[0]);
            assert_match(99, 109, text, &matches_0_distance[1]);
        }
        {
            let matches_1_distance =
                FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 1).collect::<Vec<_>>();

            assert_eq!(3, matches_1_distance.len());
            assert_match(19, 29, text, &matches_1_distance[0]);
            assert_match(42, 52, text, &matches_1_distance[1]);
            assert_match(99, 109, text, &matches_1_distance[2]);
        }
        {
            let matches_2_distance =
                FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 2).collect::<Vec<_>>();

            assert_eq!(3, matches_2_distance.len());
            assert_match(19, 29, text, &matches_2_distance[0]);
            assert_match(42, 52, text, &matches_2_distance[1]);
            assert_match(99, 109, text, &matches_2_distance[2]);
        }
    }

    #[test]
    fn no_match() {
        let pattern = "foo";
        let text = "f-------f----o---f---o-o---------------------------------------------------------------------------------------------------------------------------------------------------------";

        assert_eq!(
            0,
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 1)
                .collect::<Vec<_>>()
                .len()
        );
    }

    #[test]
    fn perfect_match() {
        let pattern = "foo-------------------------------------------------------------------------------------------------------------------------------------------------------";
        let text = "foo-------------------------------------------------------------------------------------------------------------------------------------------------------";

        let matches =
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 1).collect::<Vec<_>>();

        assert_eq!(1, matches.len());
        assert_eq!(0, matches[0].start_index);
        assert_eq!(154, matches[0].end_index);
        assert_eq!(text, matches[0].match_text);
    }

    #[test]
    fn empty_pattern() {
        let pattern = "";
        let text = "foo-------------------------------------------------------------------------------------------------------------------------------------------------------";

        assert_eq!(
            0,
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 2)
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
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 2)
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
            FuzzySearchSubstitutionsOnlySimdTest::find(pattern, text, 2)
                .collect::<Vec<_>>()
                .len()
        );
    }
}
