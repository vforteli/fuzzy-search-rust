use crate::{candidate_match::CandidateMatch, match_result::MatchResult};

pub struct MatchConsolidator<'a, TIterator: Iterator<Item = CandidateMatch>> {
    text: &'a str,
    matches: TIterator,
    max_distance: usize,
    group: Vec<CandidateMatch>,
    // match_start_index: usize,
}

impl<'a, TIterator: Iterator<Item = CandidateMatch>> MatchConsolidator<'a, TIterator> {
    pub fn consolidate(max_distance: usize, text: &'a str, matches: TIterator) -> Self {
        Self {
            text,
            matches,
            max_distance,
            group: Vec::new(),
        }
    }

    #[inline(always)]
    fn get_best_match_from_group(group: &Vec<CandidateMatch>, text: &str) -> MatchResult {
        let best_match = group
            .iter()
            .min_by(|a, b| {
                a.distance.cmp(&b.distance).then_with(|| {
                    b.start_index
                        .checked_sub(b.text_index)
                        .unwrap_or(0)
                        .cmp(&a.start_index.checked_sub(a.text_index).unwrap_or(0))
                })
            })
            .unwrap();

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
        if let Some(first_match) = self.matches.next() {
            self.group.push(first_match);

            while let Some(next_match) = self.matches.next() {
                let match_start_index = &self.group.last().unwrap().start_index; // hmm.. unwrap...
                if next_match.start_index > (match_start_index + self.max_distance) {
                    let best_match = Self::get_best_match_from_group(&self.group, self.text);

                    self.group.clear();
                    self.group.push(next_match);

                    return Some(best_match);
                }

                self.group.push(next_match);
            }
        }

        if !self.group.is_empty() {
            let best_match = Self::get_best_match_from_group(&self.group, self.text);
            self.group.clear();
            return Some(best_match);
        }

        None
    }
}
