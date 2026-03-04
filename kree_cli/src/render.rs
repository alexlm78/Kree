use std::collections::HashMap;
use std::path::Path;

use colored::{ColoredString, Colorize};

use crate::tree::TreeNode;

/// Map of file extensions to RGB color tuples.
pub type ColorMap = HashMap<String, (u8, u8, u8)>;
/// Map of file extensions/names to icon characters.
pub type IconMap = HashMap<String, String>;

/// Parses a color string (name or hex) into an RGB tuple.
///
/// Supports:
/// - Hex codes: `#RRGGBB`
/// - Standard color names: `red`, `blue`, `bright_green`, etc.
fn parse_color(name: &str) -> Option<(u8, u8, u8)> {
    if let Some(hex) = name.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some((r, g, b));
        }
        return None;
    }

    let normalized = name.replace('-', "_");
    match normalized.as_str() {
        "black" => Some((0, 0, 0)),
        "red" => Some((205, 0, 0)),
        "green" => Some((0, 205, 0)),
        "yellow" => Some((205, 205, 0)),
        "blue" => Some((0, 0, 238)),
        "magenta" => Some((205, 0, 205)),
        "cyan" => Some((0, 205, 205)),
        "white" => Some((229, 229, 229)),
        "bright_black" => Some((127, 127, 127)),
        "bright_red" => Some((255, 0, 0)),
        "bright_green" => Some((0, 255, 0)),
        "bright_yellow" => Some((255, 255, 0)),
        "bright_blue" => Some((92, 92, 255)),
        "bright_magenta" => Some((255, 0, 255)),
        "bright_cyan" => Some((0, 255, 255)),
        "bright_white" => Some((255, 255, 255)),
        _ => None,
    }
}

/// Builds a `ColorMap` by merging default colors with user-provided overrides.
///
/// Default colors cover common programming languages, configuration formats,
/// images, and archives.
pub fn build_color_map(user_colors: &HashMap<String, String>) -> ColorMap {
    let defaults: &[(&str, (u8, u8, u8))] = &[
        // Rust
        ("rs", (255, 165, 0)),
        // Web
        ("js", (205, 205, 0)),
        ("ts", (205, 205, 0)),
        ("jsx", (205, 205, 0)),
        ("tsx", (205, 205, 0)),
        ("html", (205, 0, 205)),
        ("css", (205, 0, 205)),
        ("scss", (205, 0, 205)),
        // Data / Config
        ("json", (0, 205, 205)),
        ("toml", (0, 205, 205)),
        ("yaml", (0, 205, 205)),
        ("yml", (0, 205, 205)),
        ("xml", (0, 205, 205)),
        ("csv", (0, 205, 205)),
        // Documentation
        ("md", (255, 255, 0)),
        ("txt", (255, 255, 0)),
        ("rst", (255, 255, 0)),
        // Images
        ("png", (255, 0, 255)),
        ("jpg", (255, 0, 255)),
        ("jpeg", (255, 0, 255)),
        ("gif", (255, 0, 255)),
        ("svg", (255, 0, 255)),
        ("ico", (255, 0, 255)),
        ("bmp", (255, 0, 255)),
        ("webp", (255, 0, 255)),
        // Archives
        ("zip", (205, 0, 0)),
        ("tar", (205, 0, 0)),
        ("gz", (205, 0, 0)),
        ("bz2", (205, 0, 0)),
        ("xz", (205, 0, 0)),
        ("rar", (205, 0, 0)),
        ("7z", (205, 0, 0)),
        // Lock files
        ("lock", (127, 127, 127)),
    ];

    let mut map = ColorMap::new();
    for &(ext, color) in defaults {
        map.insert(ext.to_string(), color);
    }

    for (ext, color_name) in user_colors {
        match parse_color(color_name) {
            Some(rgb) => {
                map.insert(ext.clone(), rgb);
            }
            None => {
                eprintln!(
                    "Warning: unknown color '{color_name}' for extension '{ext}' in ~/.kreerc, ignoring"
                );
            }
        }
    }

    map
}

/// Builds an `IconMap` by merging default icons with user-provided overrides.
///
/// Icons are Nerd Font characters. Defaults cover common file types.
pub fn build_icon_map(user_icons: &HashMap<String, String>) -> IconMap {
    let defaults: &[(&str, &str)] = &[
        // Special
        ("directory", "\u{f115}"),  //
        ("executable", "\u{f489}"), //
        ("default", "\u{f15b}"),    //
        // Languages
        ("rs", "\u{e7a8}"),    //
        ("py", "\u{e73c}"),    //
        ("js", "\u{e74e}"),    //
        ("ts", "\u{e628}"),    //
        ("jsx", "\u{e7ba}"),   //
        ("tsx", "\u{e7ba}"),   //
        ("go", "\u{e626}"),    //
        ("rb", "\u{e739}"),    //
        ("java", "\u{e738}"),  //
        ("c", "\u{e61e}"),     //
        ("cpp", "\u{e61d}"),   //
        ("h", "\u{e61e}"),     //
        ("hpp", "\u{e61d}"),   //
        ("lua", "\u{e620}"),   //
        ("php", "\u{e73d}"),   //
        ("swift", "\u{e755}"), //
        ("kt", "\u{e634}"),    //
        ("dart", "\u{e798}"),  //
        ("zig", "\u{e6a9}"),   //
        ("ex", "\u{e62d}"),    //
        ("hs", "\u{e61f}"),    //
        ("sh", "\u{f489}"),    //
        ("bash", "\u{f489}"),  //
        ("zsh", "\u{f489}"),   //
        ("cs", "\u{f81a}"),    //
        ("r", "\u{f25d}"),     //
        // Web
        ("html", "\u{e736}"), //
        ("css", "\u{e749}"),  //
        ("scss", "\u{e749}"), //
        ("vue", "\u{e6a0}"),  //
        // Data / Config
        ("json", "\u{e60b}"), //
        ("toml", "\u{e60b}"), //
        ("yaml", "\u{e60b}"), //
        ("yml", "\u{e60b}"),  //
        ("xml", "\u{e619}"),  //
        ("csv", "\u{f1c3}"),  //
        // Docs
        ("md", "\u{e73e}"),  //
        ("txt", "\u{f15c}"), //
        ("rst", "\u{f15c}"), //
        ("pdf", "\u{f1c1}"), //
        // Images
        ("png", "\u{f1c5}"),  //
        ("jpg", "\u{f1c5}"),  //
        ("jpeg", "\u{f1c5}"), //
        ("gif", "\u{f1c5}"),  //
        ("svg", "\u{f1c5}"),  //
        ("ico", "\u{f1c5}"),  //
        ("bmp", "\u{f1c5}"),  //
        ("webp", "\u{f1c5}"), //
        // Archives
        ("zip", "\u{f1c6}"), //
        ("tar", "\u{f1c6}"), //
        ("gz", "\u{f1c6}"),  //
        ("bz2", "\u{f1c6}"), //
        ("xz", "\u{f1c6}"),  //
        ("rar", "\u{f1c6}"), //
        ("7z", "\u{f1c6}"),  //
        // Other
        ("lock", "\u{f023}"),       //
        ("dockerfile", "\u{e7b0}"), //
        ("gitignore", "\u{e702}"),  //
    ];

    let mut map = IconMap::new();
    for &(key, icon) in defaults {
        map.insert(key.to_string(), icon.to_string());
    }

    for (key, icon) in user_icons {
        map.insert(key.clone(), icon.clone());
    }

    map
}

/// Determines the icon to use for a given file path.
///
/// Priority:
/// 1. Directory icon (if it's a directory)
/// 2. Exact extension match
/// 3. Exact filename match (e.g. `Dockerfile`)
/// 4. Executable icon (if executable)
/// 5. Default icon
pub(crate) fn icon_for_node<'a>(path: &Path, icon_map: &'a IconMap) -> &'a str {
    if path.is_dir() {
        if let Some(icon) = icon_map.get("directory") {
            return icon.as_str();
        }
        return "";
    }

    // Try extension first
    if let Some(ext) = path.extension().and_then(|e| e.to_str())
        && let Some(icon) = icon_map.get(&ext.to_lowercase())
    {
        return icon.as_str();
    }

    // Try full filename (e.g. Dockerfile, .gitignore)
    if let Some(filename) = path.file_name().and_then(|n| n.to_str())
        && let Some(icon) = icon_map.get(&filename.to_lowercase())
    {
        return icon.as_str();
    }

    // Executable check
    if is_executable(path)
        && let Some(icon) = icon_map.get("executable")
    {
        return icon.as_str();
    }

    // Default
    if let Some(icon) = icon_map.get("default") {
        return icon.as_str();
    }

    ""
}

fn colorize_name(
    name: &str,
    path: &Path,
    color_map: &ColorMap,
    icon_map: Option<&IconMap>,
) -> String {
    let colored = if path.is_dir() {
        name.blue().bold().to_string()
    } else if is_executable(path) {
        name.green().bold().to_string()
    } else {
        colorize_by_extension(name, path, color_map).to_string()
    };

    match icon_map {
        Some(im) => {
            let icon = icon_for_node(path, im);
            format!("{icon} {colored}")
        }
        None => colored,
    }
}

fn colorize_by_extension(name: &str, path: &Path, color_map: &ColorMap) -> ColoredString {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match color_map.get(&ext.to_lowercase()) {
        Some(&(r, g, b)) => name.truecolor(r, g, b),
        None => name.bright_white(),
    }
}

/// Checks if a file is executable.
///
/// On Unix-like systems, this checks the execute permission bit.
#[cfg(unix)]
pub(crate) fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| !m.is_dir() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Checks if a file is executable.
///
/// On non-Unix systems, this always returns false.
#[cfg(not(unix))]
pub(crate) fn is_executable(_path: &Path) -> bool {
    false
}

/// Renders the directory tree to stdout.
///
/// # Arguments
///
/// * `root` - The root node of the tree.
/// * `color_map` - Configuration for file colors.
/// * `icon_map` - Optional configuration for file icons.
pub fn render_tree(root: &TreeNode, color_map: &ColorMap, icon_map: Option<&IconMap>) {
    println!(
        "└── {}",
        colorize_name(&root.name, &root.path, color_map, icon_map)
    );
    let child_count = root.children.len();
    for (i, child) in root.children.iter().enumerate() {
        let is_last = i == child_count - 1;
        let mask = if is_last { 0b11u64 } else { 0b01u64 };
        render_node(child, 1, is_last, mask, color_map, icon_map);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // parse_color tests

    #[test]
    fn parse_hex_color() {
        assert_eq!(parse_color("#FF6600"), Some((255, 102, 0)));
    }

    #[test]
    fn parse_named_color() {
        assert_eq!(parse_color("red"), Some((205, 0, 0)));
    }

    #[test]
    fn parse_hyphenated_color() {
        assert_eq!(parse_color("bright-red"), Some((255, 0, 0)));
    }

    #[test]
    fn parse_invalid_color() {
        assert_eq!(parse_color("nope"), None);
    }

    #[test]
    fn parse_short_hex_invalid() {
        assert_eq!(parse_color("#FFF"), None);
    }

    // build_color_map tests

    #[test]
    fn color_map_has_defaults() {
        let map = build_color_map(&HashMap::new());
        assert_eq!(map.get("rs"), Some(&(255, 165, 0)));
    }

    #[test]
    fn color_map_user_override() {
        let mut user = HashMap::new();
        user.insert("rs".to_string(), "#00FF00".to_string());
        let map = build_color_map(&user);
        assert_eq!(map.get("rs"), Some(&(0, 255, 0)));
    }

    // build_icon_map tests

    #[test]
    fn icon_map_has_defaults() {
        let map = build_icon_map(&HashMap::new());
        assert!(map.contains_key("rs"));
    }

    #[test]
    fn icon_map_user_override() {
        let mut user = HashMap::new();
        user.insert("rs".to_string(), "X".to_string());
        let map = build_icon_map(&user);
        assert_eq!(map.get("rs"), Some(&"X".to_string()));
    }

    // is_executable tests (unix only)

    #[cfg(unix)]
    #[test]
    fn executable_file_detected() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("script.sh");
        std::fs::write(&file_path, "#!/bin/sh").unwrap();
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(is_executable(&file_path));
    }

    #[cfg(unix)]
    #[test]
    fn non_executable_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("data.txt");
        std::fs::write(&file_path, "hello").unwrap();
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(!is_executable(&file_path));
    }
}

fn render_node(
    node: &TreeNode,
    depth: u32,
    is_last: bool,
    mask: u64,
    color_map: &ColorMap,
    icon_map: Option<&IconMap>,
) {
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

    println!(
        "{}",
        colorize_name(&node.name, &node.path, color_map, icon_map)
    );

    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i == child_count - 1;
        let new_mask = if child_is_last {
            mask | (1u64 << (depth + 1))
        } else {
            mask
        };
        render_node(
            child,
            depth + 1,
            child_is_last,
            new_mask,
            color_map,
            icon_map,
        );
    }
}
