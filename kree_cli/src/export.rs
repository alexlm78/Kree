use std::time::SystemTime;

use serde::Serialize;

use crate::tree::TreeNode;

/// Serializable representation of a tree node.
#[derive(Serialize)]
struct ExportNode {
    name: String,
    path: String,
    #[serde(rename = "type")]
    node_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(unix)]
    permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(unix)]
    owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symlink_target: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<ExportNode>,
}

fn to_export_node(node: &TreeNode) -> ExportNode {
    let node_type = if node.path.is_dir() {
        "directory"
    } else if node.is_symlink {
        "symlink"
    } else {
        "file"
    };

    let (size, modified) = if let Some(ref meta) = node.metadata {
        (meta.size, meta.modified.as_ref().map(format_iso_time))
    } else {
        (None, None)
    };

    #[cfg(unix)]
    let (permissions, owner) = if let Some(ref meta) = node.metadata {
        (meta.mode.map(format_mode), meta.owner.clone())
    } else {
        (None, None)
    };

    ExportNode {
        name: node.name.clone(),
        path: node.path.display().to_string(),
        node_type,
        size,
        modified,
        #[cfg(unix)]
        permissions,
        #[cfg(unix)]
        owner,
        symlink_target: node.symlink_target.as_ref().map(|p| p.display().to_string()),
        children: node.children.iter().map(to_export_node).collect(),
    }
}

fn format_iso_time(time: &SystemTime) -> String {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs() as i64;
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    let (year, month, day) = days_to_ymd(days);
    format!("{year}-{:02}-{:02}T{hours:02}:{minutes:02}:{seconds:02}Z", month + 1, day)
}

fn days_to_ymd(mut days: i64) -> (i64, i64, i64) {
    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m as i64 - 1, d as i64)
}

#[cfg(unix)]
fn format_mode(mode: u32) -> String {
    let flags = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    flags.iter().map(|&(bit, ch)| if mode & bit != 0 { ch } else { '-' }).collect()
}

/// Exports the tree as JSON string.
pub fn export_json(node: &TreeNode) -> String {
    let export = to_export_node(node);
    serde_json::to_string_pretty(&export).unwrap_or_else(|e| format!("Error: {e}"))
}

/// Exports the tree as YAML string.
pub fn export_yaml(node: &TreeNode) -> String {
    let export = to_export_node(node);
    serde_yml::to_string(&export).unwrap_or_else(|e| format!("Error: {e}"))
}

/// Exports the tree as a Markdown indented list.
pub fn export_markdown(node: &TreeNode) -> String {
    let mut output = String::new();
    output.push_str(&format!("- **{}**\n", node.name));
    for child in &node.children {
        write_markdown(child, 1, &mut output);
    }
    output
}

fn write_markdown(node: &TreeNode, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    if node.path.is_dir() {
        output.push_str(&format!("{indent}- **{}**/\n", node.name));
    } else {
        output.push_str(&format!("{indent}- {}\n", node.name));
    }
    for child in &node.children {
        write_markdown(child, depth + 1, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::TreeNode;
    use std::path::PathBuf;

    fn sample_tree() -> TreeNode {
        TreeNode {
            name: "root".to_string(),
            path: PathBuf::from("/tmp/root"),
            is_symlink: false,
            symlink_target: None,
            metadata: None,
            children: vec![
                TreeNode {
                    name: "file.txt".to_string(),
                    path: PathBuf::from("/tmp/root/file.txt"),
                    is_symlink: false,
                    symlink_target: None,
                    metadata: None,
                    children: vec![],
                },
            ],
        }
    }

    #[test]
    fn json_export_contains_name() {
        let json = export_json(&sample_tree());
        assert!(json.contains("\"name\": \"root\""));
        assert!(json.contains("\"name\": \"file.txt\""));
    }

    #[test]
    fn yaml_export_contains_name() {
        let yaml = export_yaml(&sample_tree());
        assert!(yaml.contains("name: root"));
        assert!(yaml.contains("name: file.txt"));
    }

    #[test]
    fn markdown_export_structure() {
        let md = export_markdown(&sample_tree());
        assert!(md.contains("- **root**"));
        assert!(md.contains("  - file.txt"));
    }
}
