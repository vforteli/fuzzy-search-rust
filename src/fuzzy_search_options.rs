pub struct FuzzySearchOptions {
    pub max_total_distance: usize,
    pub max_substitutions: usize,
    pub max_deletions: usize,
    pub max_insertions: usize,
}

impl FuzzySearchOptions {
    pub fn new(max_total_distance: usize) -> Self {
        FuzzySearchOptions {
            max_total_distance,
            max_substitutions: max_total_distance,
            max_deletions: max_total_distance,
            max_insertions: max_total_distance,
        }
    }

    pub fn with_limits(
        max_total_distance: usize,
        max_substitutions: Option<usize>,
        max_deletions: Option<usize>,
        max_insertions: Option<usize>,
    ) -> Self {
        let max_substitutions = max_substitutions.unwrap_or(max_total_distance);
        let max_deletions = max_deletions.unwrap_or(max_total_distance);
        let max_insertions = max_insertions.unwrap_or(max_total_distance);

        FuzzySearchOptions {
            max_total_distance,
            max_substitutions,
            max_deletions,
            max_insertions,
        }
    }

    pub fn with_individual_limits(
        max_substitutions: usize,
        max_deletions: usize,
        max_insertions: usize,
    ) -> Self {
        FuzzySearchOptions {
            max_substitutions,
            max_deletions,
            max_insertions,
            max_total_distance: max_deletions + max_insertions + max_substitutions,
        }
    }

    pub fn can_substitute(
        &self,
        current_total_distance: usize,
        current_substitutions: usize,
    ) -> bool {
        current_substitutions < self.max_substitutions
            && current_total_distance < self.max_total_distance
    }

    pub fn can_delete(&self, current_total_distance: usize, current_deletions: usize) -> bool {
        current_deletions < self.max_deletions && current_total_distance < self.max_total_distance
    }

    pub fn can_insert(&self, current_total_distance: usize, current_insertions: usize) -> bool {
        current_insertions < self.max_insertions && current_total_distance < self.max_total_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_substitute_total_distance() {
        let options = FuzzySearchOptions::new(3);

        assert!(!options.can_substitute(3, 1));
        assert!(options.can_substitute(2, 1));
    }

    #[test]
    fn test_can_substitute_total_and_max_distance() {
        let options = FuzzySearchOptions::with_limits(3, Some(1), Some(1), Some(1));

        assert!(!options.can_substitute(2, 1));
        assert!(options.can_substitute(2, 0));
    }

    #[test]
    fn test_can_substitute_max_substitutions() {
        let options = FuzzySearchOptions::with_individual_limits(1, 1, 1);

        assert!(!options.can_substitute(100, 1));
        assert!(options.can_substitute(0, 0));
    }

    #[test]
    fn test_can_delete_total_distance() {
        let options = FuzzySearchOptions::new(3);

        assert!(!options.can_delete(3, 1));
        assert!(options.can_delete(2, 1));
    }

    #[test]
    fn test_can_delete_total_and_max_distance() {
        let options = FuzzySearchOptions::with_limits(3, Some(1), Some(1), Some(1));

        assert!(!options.can_delete(2, 1));
        assert!(options.can_delete(2, 0));
    }

    #[test]
    fn test_can_delete_max_substitutions() {
        let options = FuzzySearchOptions::with_individual_limits(1, 1, 1);

        assert!(!options.can_delete(100, 1));
        assert!(options.can_delete(0, 0));
    }

    #[test]
    fn test_can_insert_total_distance() {
        let options = FuzzySearchOptions::new(3);

        assert!(!options.can_insert(3, 1));
        assert!(options.can_insert(2, 1));
    }

    #[test]
    fn test_can_insert_total_and_max_distance() {
        let options = FuzzySearchOptions::with_limits(3, Some(1), Some(1), Some(1));

        assert!(!options.can_insert(2, 1));
        assert!(options.can_insert(2, 0));
    }

    #[test]
    fn test_can_insert_max_substitutions() {
        let options = FuzzySearchOptions::with_individual_limits(1, 1, 1);

        assert!(!options.can_insert(100, 1));
        assert!(options.can_insert(0, 0));
    }
}
