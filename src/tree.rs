use std::fs;
use std::path::PathBuf;

use clap::ValueEnum;

use crate::ignore::IgnoreFilter;

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
pub fn load_tree(root: &PathBuf, max_depth: u32, current_depth: u32, filter: &IgnoreFilter, sort: SortMode) -> TreeNode {
    let name = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned());

    let mut node = TreeNode {
        name,
        path: root.clone(),
        children: Vec::new(),
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
        let child = load_tree(&child_path, max_depth, current_depth + 1, filter, sort);
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
                b_is_dir.cmp(&a_is_dir)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
        }
    }
    node.children = children;

    node
}
