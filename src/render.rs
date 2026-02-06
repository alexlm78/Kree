use std::path::Path;

use colored::Colorize;

use crate::tree::TreeNode;

fn colorize_name(name: &str, path: &Path) -> String {
    if path.is_dir() {
        name.blue().bold().to_string()
    } else if is_executable(path) {
        name.green().bold().to_string()
    } else {
        name.bright_white().to_string()
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| !m.is_dir() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    false
}

pub fn render_tree(root: &TreeNode) {
    println!("└── {}", colorize_name(&root.name, &root.path));
    let child_count = root.children.len();
    for (i, child) in root.children.iter().enumerate() {
        let is_last = i == child_count - 1;
        let mask = if is_last { 0b11u64 } else { 0b01u64 };
        render_node(child, 1, is_last, mask);
    }
}

fn render_node(node: &TreeNode, depth: u32, is_last: bool, mask: u64) {
    for i in 0..depth {
        if ((mask >> i) & 1) == 0 {
            print!("│    ");
        } else {
            print!("     ");
        }
    }

    if is_last {
        print!("└── ");
    } else {
        print!("├── ");
    }

    println!("{}", colorize_name(&node.name, &node.path));

    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i == child_count - 1;
        let new_mask = if child_is_last {
            mask | (1u64 << (depth + 1))
        } else {
            mask
        };
        render_node(child, depth + 1, child_is_last, new_mask);
    }
}
