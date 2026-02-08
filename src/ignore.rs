use std::collections::HashSet;
use std::fs;

/// Filter for ignoring files and directories during tree traversal.
///
/// It handles ignoring files based on:
/// 1. Hidden files (starting with `.`)
/// 2. Patterns listed in `.kreeignore` file in the current directory
/// 3. Additional patterns passed from configuration or arguments
#[derive(Clone)]
pub struct IgnoreFilter {
    excluded: HashSet<String>,
    active: bool,
}

impl IgnoreFilter {
    /// Creates a new `IgnoreFilter`.
    ///
    /// # Arguments
    ///
    /// * `active` - Whether filtering is enabled. If false, no files are ignored.
    /// * `config_patterns` - Additional patterns to ignore from configuration.
    ///
    /// If `active` is true, it attempts to read `.kreeignore` from the current directory
    /// and adds those patterns to the exclusion list.
    pub fn new(active: bool, config_patterns: &[String]) -> Self {
        if !active {
            return IgnoreFilter {
                excluded: HashSet::new(),
                active: false,
            };
        }

        let mut excluded: HashSet<String> = match fs::read_to_string(".kreeignore") {
            Ok(contents) => contents
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect(),
            Err(_) => HashSet::new(),
        };

        for pattern in config_patterns {
            excluded.insert(pattern.clone());
        }

        IgnoreFilter {
            excluded,
            active: true,
        }
    }

    /// Checks if a given filename should be ignored.
    ///
    /// Returns `true` if:
    /// - Filtering is active AND
    /// - (Filename starts with `.` OR Filename matches an excluded pattern)
    pub fn is_ignored(&self, filename: &str) -> bool {
        if !self.active {
            return false;
        }
        filename.starts_with('.') || self.excluded.contains(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_filter_ignores_nothing() {
        let filter = IgnoreFilter::new(false, &["target".to_string()]);
        assert!(!filter.is_ignored("target"));
    }

    #[test]
    fn active_filter_hides_dotfiles() {
        let filter = IgnoreFilter::new(true, &[]);
        assert!(filter.is_ignored(".git"));
        assert!(filter.is_ignored(".env"));
    }

    #[test]
    fn active_filter_matches_patterns() {
        let filter = IgnoreFilter::new(true, &["target".to_string(), "node_modules".to_string()]);
        assert!(filter.is_ignored("target"));
        assert!(filter.is_ignored("node_modules"));
    }

    #[test]
    fn active_filter_allows_normal_files() {
        let filter = IgnoreFilter::new(true, &["target".to_string()]);
        assert!(!filter.is_ignored("main.rs"));
        assert!(!filter.is_ignored("Cargo.toml"));
    }

    #[test]
    fn inactive_filter_allows_dotfiles() {
        let filter = IgnoreFilter::new(false, &[]);
        assert!(!filter.is_ignored(".git"));
        assert!(!filter.is_ignored(".env"));
    }
}
