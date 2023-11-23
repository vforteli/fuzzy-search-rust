#[derive(Debug)]
pub struct CandidateMatch {
    pub start_index: usize,
    pub text_index: usize,
    pub pattern_index: usize,
    pub distance: usize,
    pub deletions: usize,
    pub substitutions: usize,
    pub insertions: usize,
}

impl CandidateMatch {
    pub fn new(start_index: usize, text_index: usize) -> Self {
        CandidateMatch {
            start_index,
            text_index,
            pattern_index: 0,
            distance: 0,
            deletions: 0,
            substitutions: 0,
            insertions: 0,
        }
    }
}
