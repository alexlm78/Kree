use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::path::Path;
use tempfile::TempDir;

/// Integration tests for Kree CLI.
///
/// These tests verify the behavior of the `kree` binary by creating temporary
/// directory structures and asserting the output of the command.

/// Helper function to create a file with empty content.
///
/// # Arguments
///
/// * `dir` - The parent directory.
/// * `name` - The name of the file to create.
fn create_file(dir: &Path, name: &str) {
    let path = dir.join(name);
    File::create(path).expect("Failed to create file");
}

/// Helper function to create a directory.
///
/// # Arguments
///
/// * `dir` - The parent directory.
/// * `name` - The name of the directory to create.
///
/// # Returns
///
/// * `PathBuf` - The path to the created directory.
fn create_dir(dir: &Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::create_dir(&path).expect("Failed to create directory");
    path
}

/// Verifies that Kree can render a basic directory tree.
///
/// This test creates a temporary directory with a known structure:
/// ```text
/// root/
/// ├── file1.txt
/// └── subdir/
///     └── file2.rs
/// ```
/// It asserts that the output contains the filenames and directory names.
#[test]
fn test_basic_tree_rendering() {
    // 1. Setup: Create temporary directory and files
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = temp.path();

    create_file(root, "file1.txt");
    let subdir = create_dir(root, "subdir");
    create_file(&subdir, "file2.rs");

    // 2. Execution: Run `kree` with `--no-color` to simplify output matching
    // We target the binary "kree" defined in Cargo.toml
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kree"));
    cmd.arg(root.to_str().unwrap())
       .arg("--no-color");

    // 3. Assertion: Verify output contains key elements
    // Note: We use predicates to check for substrings because the exact tree characters
    // (├──, └──) might depend on implementation details or environment.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("subdir"))
        .stdout(predicate::str::contains("file2.rs"));
}

/// Verifies that the `--depth` argument correctly limits traversal.
///
/// Structure:
/// ```text
/// root/
/// ├── level1/
/// │   └── level2/
/// │       └── file_deep.txt
/// ```
/// With depth 1, `level2` and `file_deep.txt` should not be shown.
#[test]
fn test_depth_limit() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = temp.path();

    let level1 = create_dir(root, "level1");
    let level2 = create_dir(&level1, "level2");
    create_file(&level2, "file_deep.txt");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kree"));
    cmd.arg(root.to_str().unwrap())
       .arg("--depth")
       .arg("1")
       .arg("--no-color");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("level1"))
        .stdout(predicate::str::contains("level2").not())
        .stdout(predicate::str::contains("file_deep.txt").not());
}

/// Verifies that hidden files are ignored by default and shown with `--all`.
#[test]
fn test_hidden_files() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = temp.path();

    create_file(root, ".hidden_file");
    create_file(root, "visible_file");

    // Case 1: Default behavior (hidden files ignored)
    let mut cmd_default = Command::new(env!("CARGO_BIN_EXE_kree"));
    cmd_default.arg(root.to_str().unwrap())
               .arg("--no-color");
    
    cmd_default.assert()
        .success()
        .stdout(predicate::str::contains("visible_file"))
        .stdout(predicate::str::contains(".hidden_file").not());

    // Case 2: With `--all` (show hidden files)
    let mut cmd_all = Command::new(env!("CARGO_BIN_EXE_kree"));
    cmd_all.arg(root.to_str().unwrap())
           .arg("--all")
           .arg("--no-color");
    
    cmd_all.assert()
        .success()
        .stdout(predicate::str::contains("visible_file"))
        .stdout(predicate::str::contains(".hidden_file"));
}
