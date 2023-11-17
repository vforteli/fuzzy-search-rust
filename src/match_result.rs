#[derive(Debug, Clone)]
pub struct MatchResult {
    pub start_index: usize,
    pub end_index: usize,
    pub distance: usize,
    pub match_text: String,
    pub deletions: usize,
    pub substitutions: usize,
    pub insertions: usize,
}
