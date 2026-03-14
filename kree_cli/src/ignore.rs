use std::fs;
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

/// Filter for ignoring files and directories during tree traversal.
///
/// It handles ignoring files based on:
/// 1. Hidden files (starting with `.`)
/// 2. Glob patterns listed in `.kreeignore` file in the current directory
/// 3. Additional glob patterns passed from configuration or arguments
///
/// Supports full glob syntax: `*.log`, `build_*`, `**/*.tmp`, `target`, etc.
#[derive(Clone)]
pub struct IgnoreFilter {
    globs: GlobSet,
    active: bool,
}

impl IgnoreFilter {
    /// Creates a new `IgnoreFilter`.
    ///
    /// # Arguments
    ///
    /// * `active` - Whether filtering is enabled. If false, no files are ignored.
    /// * `config_patterns` - Additional glob patterns to ignore from configuration.
    ///
    /// If `active` is true, it attempts to read `.kreeignore` from the current directory
    /// and compiles all patterns (file + config) into a `GlobSet`.
    /// Invalid glob patterns are silently skipped with a warning.
    pub fn new(active: bool, config_patterns: &[String]) -> Self {
        if !active {
            return IgnoreFilter {
                globs: GlobSet::empty(),
                active: false,
            };
        }

        let file_patterns: Vec<String> = match fs::read_to_string(".kreeignore") {
            Ok(contents) => contents
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
                .map(|line| line.trim().to_string())
                .collect(),
            Err(_) => Vec::new(),
        };

        let mut builder = GlobSetBuilder::new();
        for pattern in file_patterns.iter().chain(config_patterns.iter()) {
            match Glob::new(pattern) {
                Ok(glob) => {
                    builder.add(glob);
                }
                Err(e) => {
                    eprintln!("Warning: invalid ignore pattern '{pattern}': {e}");
                }
            }
        }

        let globs = builder.build().unwrap_or(GlobSet::empty());
        IgnoreFilter {
            globs,
            active: true,
        }
    }

    /// Checks if a given file path should be ignored.
    ///
    /// Matches against:
    /// - The filename component (e.g. `*.log` matches `error.log`)
    /// - The full relative path (e.g. `src/*.rs` matches `src/main.rs`)
    ///
    /// Also hides dotfiles (filenames starting with `.`) when active.
    pub fn is_ignored(&self, filename: &str) -> bool {
        self.is_ignored_path(Path::new(filename))
    }

    /// Checks if a path should be ignored, matching against both
    /// filename and full path components.
    pub fn is_ignored_path(&self, path: &Path) -> bool {
        if !self.active {
            return false;
        }
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if filename.starts_with('.') {
            return true;
        }

        self.globs.is_match(filename) || self.globs.is_match(path)
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
    fn active_filter_matches_exact_name() {
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

    #[test]
    fn glob_wildcard_extension() {
        let filter = IgnoreFilter::new(true, &["*.log".to_string()]);
        assert!(filter.is_ignored("error.log"));
        assert!(filter.is_ignored("server.log"));
        assert!(!filter.is_ignored("main.rs"));
    }

    #[test]
    fn glob_prefix_wildcard() {
        let filter = IgnoreFilter::new(true, &["build_*".to_string()]);
        assert!(filter.is_ignored("build_debug"));
        assert!(filter.is_ignored("build_release"));
        assert!(!filter.is_ignored("target"));
    }

    #[test]
    fn glob_question_mark() {
        let filter = IgnoreFilter::new(true, &["file?.txt".to_string()]);
        assert!(filter.is_ignored("file1.txt"));
        assert!(filter.is_ignored("fileA.txt"));
        assert!(!filter.is_ignored("file10.txt"));
    }
}
