use crate::candidate_match::CandidateMatch;

pub struct MatchConsolidator<TIterator: Iterator<Item = CandidateMatch>> {
    matches: TIterator,
    max_distance: usize,
    group: Vec<CandidateMatch>,
}

impl<'a, TIterator: Iterator<Item = CandidateMatch>> MatchConsolidator<TIterator> {
    pub fn consolidate(max_distance: usize, matches: TIterator) -> Self {
        Self {
            matches,
            max_distance,
            group: Vec::new(),
        }
    }

    #[inline(always)]
    fn get_best_match_from_group(group: &Vec<CandidateMatch>) -> CandidateMatch {
        group
            .iter()
            .min_by(|a, b| {
                a.distance.cmp(&b.distance).then_with(|| {
                    (b.text_index - b.start_index).cmp(&(a.text_index - a.start_index))
                })
            })
            .expect("uh, why no candidate match?")
            .clone()
    }
}

impl<'a, TIterator: Iterator<Item = CandidateMatch>> Iterator for MatchConsolidator<TIterator> {
    type Item = CandidateMatch;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first_match) = self.matches.next() {
            self.group.push(first_match);

            while let Some(next_match) = self.matches.next() {
                let match_start_index = &self.group.last().unwrap().start_index; // hmm.. unwrap...
                if next_match.start_index > (match_start_index + self.max_distance) {
                    let best_match = Self::get_best_match_from_group(&self.group);

                    self.group.clear();
                    self.group.push(next_match);

                    return Some(best_match);
                }

                self.group.push(next_match);
            }
        }

        if !self.group.is_empty() {
            let best_match = Self::get_best_match_from_group(&self.group);
            self.group.clear();
            return Some(best_match);
        }

        None
    }
}
