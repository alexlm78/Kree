use std::fs;
use std::path::PathBuf;

use crate::ignore::IgnoreFilter;

pub struct TreeNode {
    pub name: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub children: Vec<TreeNode>,
}

pub fn load_tree(root: &PathBuf, max_depth: u32, current_depth: u32, filter: &IgnoreFilter) -> TreeNode {
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
        let child = load_tree(&child_path, max_depth, current_depth + 1, filter);
        children.push(child);
    }

    children.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    node.children = children;

    node
}
