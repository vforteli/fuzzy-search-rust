use crate::{candidate_match::CandidateMatch, match_result::MatchResult};

pub struct MatchConsolidator<'a, TIterator> {
    text: &'a str,
    matches: &'a mut TIterator,
    max_distance: usize,
}

impl<'a, TIterator: 'a> MatchConsolidator<'a, TIterator> {
    pub fn consolidate(max_distance: usize, text: &'a str, matches: &'a mut TIterator) -> Self
    where
        TIterator: Iterator<Item = CandidateMatch>,
    {
        Self {
            text,
            matches,
            max_distance,
        }
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

impl<'a, TIterator: Iterator<Item = CandidateMatch>> Iterator for MatchConsolidator<'a, TIterator> {
    type Item = MatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        let mut group = Vec::new();

        if let Some(first_match) = self.matches.next() {
            group.push(first_match);

            let mut match_start_index = first_match.start_index;

            while let Some(next_match) = self.matches.next() {
                if next_match.start_index > (match_start_index + self.max_distance) {
                    if !group.is_empty() {
                        // results.push(Self::get_best_match_from_group(&group, text));
                        group.clear();
                    }
                }

                group.push(next_match);
                match_start_index = next_match.start_index;
            }
        }

        if !group.is_empty() {
            return Some(Self::get_best_match_from_group(&group, self.text));
        }

        None
    }
}
