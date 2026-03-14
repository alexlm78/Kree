use std::fs;
use std::path::PathBuf;

use clap::ValueEnum;

use crate::ignore::IgnoreFilter;

/// Options controlling which entries are included in the tree traversal.
#[derive(Clone, Default)]
pub struct TreeOptions {
    /// Show only directories, excluding all files.
    pub dirs_only: bool,
    /// If non-empty, only show files whose extension matches one of these (lowercase, no dot).
    /// Directories are always shown to preserve tree structure.
    pub extensions: Vec<String>,
}

/// Specifies how entries should be sorted in the tree.
#[derive(Clone, Copy, ValueEnum)]
pub enum SortMode {
    /// Sort alphabetically, directories and files mixed.
    Name,
    /// Directories first, then files, each group sorted alphabetically.
    Kind,
}

/// Represents a node in the directory tree.
pub struct TreeNode {
    /// Name of the file or directory.
    pub name: String,
    /// Full path to the node.
    pub path: PathBuf,
    /// List of children nodes (empty for files).
    pub children: Vec<TreeNode>,
    /// True if this node is a symbolic link.
    pub is_symlink: bool,
    /// The target path of the symlink, if applicable.
    pub symlink_target: Option<PathBuf>,
}

/// Builds a tree structure from the filesystem starting at the given root.
///
/// This function recursively traverses the directory structure up to `max_depth`.
/// It applies the provided `IgnoreFilter` and sorts children according to `SortMode`.
///
/// # Arguments
///
/// * `root` - The root directory path.
/// * `max_depth` - Maximum recursion depth.
/// * `current_depth` - Current recursion depth (start with 0).
/// * `filter` - Filter for ignoring files/directories.
/// * `sort` - Sorting strategy for children.
/// * `opts` - Additional traversal options (dirs-only, etc.).
pub fn load_tree(
    root: &PathBuf,
    max_depth: u32,
    current_depth: u32,
    filter: &IgnoreFilter,
    sort: SortMode,
    opts: &TreeOptions,
) -> TreeNode {
    let name = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned());

    let (is_symlink, symlink_target) = {
        let meta = fs::symlink_metadata(root);
        let symlink = meta
            .as_ref()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        let target = if symlink {
            fs::read_link(root).ok()
        } else {
            None
        };
        (symlink, target)
    };

    let mut node = TreeNode {
        name,
        path: root.clone(),
        children: Vec::new(),
        is_symlink,
        symlink_target,
    };

    if current_depth >= max_depth {
        return node;
    }

    if !root.is_dir() {
        return node;
    }

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return node,
    };

    let mut children: Vec<TreeNode> = Vec::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().into_owned();

        if filter.is_ignored(&file_name) {
            continue;
        }

        let child_path = entry.path();

        if opts.dirs_only && !child_path.is_dir() {
            continue;
        }

        // Extension filter: skip files whose extension is not in the list.
        // Directories always pass through to preserve tree structure.
        if !opts.extensions.is_empty() && !child_path.is_dir() {
            let ext = child_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !opts.extensions.contains(&ext) {
                continue;
            }
        }

        let child = load_tree(
            &child_path,
            max_depth,
            current_depth + 1,
            filter,
            sort,
            opts,
        );
        children.push(child);
    }

    match sort {
        SortMode::Name => {
            children.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
        SortMode::Kind => {
            children.sort_by(|a, b| {
                let a_is_dir = a.path.is_dir();
                let b_is_dir = b.path.is_dir();
                b_is_dir
                    .cmp(&a_is_dir)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
        }
    }
    node.children = children;

    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_tree() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        // Create files
        fs::write(dir.path().join("banana.txt"), "").unwrap();
        fs::write(dir.path().join("apple.txt"), "").unwrap();
        // Create subdirectories
        fs::create_dir(dir.path().join("cherry")).unwrap();
        fs::write(dir.path().join("cherry").join("inner.txt"), "").unwrap();
        fs::create_dir(dir.path().join("avocado")).unwrap();
        // Create an ignored entry
        fs::create_dir(dir.path().join("excluded_dir")).unwrap();
        dir
    }

    #[test]
    fn sort_name_mixes_dirs_and_files() {
        let dir = setup_tree();
        let filter = IgnoreFilter::new(false, &[]);
        let tree = load_tree(
            &dir.path().to_path_buf(),
            1,
            0,
            &filter,
            SortMode::Name,
            &TreeOptions::default(),
        );
        let names: Vec<&str> = tree.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "apple.txt",
                "avocado",
                "banana.txt",
                "cherry",
                "excluded_dir"
            ]
        );
    }

    #[test]
    fn sort_kind_dirs_first() {
        let dir = setup_tree();
        let filter = IgnoreFilter::new(false, &[]);
        let tree = load_tree(
            &dir.path().to_path_buf(),
            1,
            0,
            &filter,
            SortMode::Kind,
            &TreeOptions::default(),
        );
        let names: Vec<&str> = tree.children.iter().map(|c| c.name.as_str()).collect();
        // Dirs (avocado, cherry, excluded_dir) come first, then files (apple.txt, banana.txt)
        assert_eq!(
            names,
            vec![
                "avocado",
                "cherry",
                "excluded_dir",
                "apple.txt",
                "banana.txt"
            ]
        );
    }

    #[test]
    fn depth_zero_returns_no_children() {
        let dir = setup_tree();
        let filter = IgnoreFilter::new(false, &[]);
        let tree = load_tree(
            &dir.path().to_path_buf(),
            0,
            0,
            &filter,
            SortMode::Name,
            &TreeOptions::default(),
        );
        assert!(tree.children.is_empty());
    }

    #[test]
    fn filter_excludes_ignored_entries() {
        let dir = setup_tree();
        let filter = IgnoreFilter::new(true, &["excluded_dir".to_string()]);
        let tree = load_tree(
            &dir.path().to_path_buf(),
            1,
            0,
            &filter,
            SortMode::Name,
            &TreeOptions::default(),
        );
        let names: Vec<&str> = tree.children.iter().map(|c| c.name.as_str()).collect();
        assert!(!names.contains(&"excluded_dir"));
        assert!(names.contains(&"apple.txt"));
    }
}
