use std::fs;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Filter for ignoring files and directories during tree traversal.
///
/// It handles ignoring files based on:
/// 1. Hidden files (starting with `.`)
/// 2. Glob patterns listed in `.kreeignore` file in the current directory
/// 3. Additional glob patterns passed from configuration or arguments
/// 4. `.gitignore` rules (when enabled)
///
/// Supports full glob syntax: `*.log`, `build_*`, `**/*.tmp`, `target`, etc.
#[derive(Clone)]
pub struct IgnoreFilter {
    globs: GlobSet,
    gitignore: Option<Gitignore>,
    root: PathBuf,
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
    #[cfg(test)]
    pub fn new(active: bool, config_patterns: &[String]) -> Self {
        Self::with_gitignore(active, config_patterns, true, Path::new("."))
    }

    /// Creates a new `IgnoreFilter` with explicit gitignore control.
    ///
    /// # Arguments
    ///
    /// * `active` - Whether filtering is enabled.
    /// * `config_patterns` - Additional glob patterns to ignore from configuration.
    /// * `use_gitignore` - Whether to load and apply `.gitignore` rules.
    /// * `root` - The root directory for resolving `.gitignore` paths.
    pub fn with_gitignore(
        active: bool,
        config_patterns: &[String],
        use_gitignore: bool,
        root: &Path,
    ) -> Self {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        if !active {
            return IgnoreFilter {
                globs: GlobSet::empty(),
                gitignore: None,
                root,
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

        let gitignore = if use_gitignore {
            Self::load_gitignore(&root)
        } else {
            None
        };

        IgnoreFilter {
            globs,
            gitignore,
            root,
            active: true,
        }
    }

    /// Walks up from `root` to find the git repository root, then loads
    /// all `.gitignore` files from repo root down to `root`.
    fn load_gitignore(root: &Path) -> Option<Gitignore> {
        // Find the git repo root by walking up
        let repo_root = Self::find_git_root(root)?;

        let mut builder = GitignoreBuilder::new(&repo_root);

        // Add the repo root .gitignore
        let root_gitignore = repo_root.join(".gitignore");
        if root_gitignore.exists() {
            builder.add(&root_gitignore);
        }

        // If root differs from repo_root, also load .gitignore in root
        if root != repo_root {
            let local_gitignore = root.join(".gitignore");
            if local_gitignore.exists() {
                builder.add(&local_gitignore);
            }
        }

        builder.build().ok()
    }

    /// Finds the git repository root by looking for a `.git` directory.
    fn find_git_root(start: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();
        loop {
            if current.join(".git").exists() {
                return Some(current);
            }
            if !current.pop() {
                return None;
            }
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

        if self.globs.is_match(filename) || self.globs.is_match(path) {
            return true;
        }

        // Check gitignore rules
        if let Some(ref gi) = self.gitignore {
            let full_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.root.join(path)
            };
            let is_dir = full_path.is_dir();
            if gi.matched(&full_path, is_dir).is_ignore() {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

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

    #[test]
    fn gitignore_is_respected() {
        let dir = tempdir().unwrap();
        // Create a .git directory to mark as repo root
        fs::create_dir(dir.path().join(".git")).unwrap();
        // Create a .gitignore
        fs::write(dir.path().join(".gitignore"), "*.tmp\nbuild/\n").unwrap();
        // Create files
        fs::write(dir.path().join("test.tmp"), "").unwrap();
        fs::create_dir(dir.path().join("build")).unwrap();
        fs::write(dir.path().join("keep.rs"), "").unwrap();

        let filter = IgnoreFilter::with_gitignore(true, &[], true, dir.path());
        assert!(filter.is_ignored_path(&dir.path().join("test.tmp")));
        assert!(filter.is_ignored_path(&dir.path().join("build")));
        assert!(!filter.is_ignored_path(&dir.path().join("keep.rs")));
    }

    #[test]
    fn gitignore_disabled() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".gitignore"), "*.tmp\n").unwrap();
        fs::write(dir.path().join("test.tmp"), "").unwrap();

        let filter = IgnoreFilter::with_gitignore(true, &[], false, dir.path());
        assert!(!filter.is_ignored_path(&dir.path().join("test.tmp")));
    }
}
