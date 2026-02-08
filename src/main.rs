//! Kree - A Directory Tree Visualizer and Fuzzy Finder
//!
//! Kree is a command-line tool that provides a tree-like visualization of directory structures,
//! similar to the standard `tree` command, but with additional features like fuzzy searching
//! and an interactive TUI (Text User Interface) mode.
//!
//! # Features
//! - Directory tree visualization with customizable depth
//! - Fuzzy search for finding files and directories quickly
//! - Interactive TUI mode for navigating the file system
//! - Support for ignoring files (via `.kreeignore` and `.gitignore` conventions)
//! - Syntax highlighting and file icons (via Nerd Fonts)
//!
//! # Modules
//! - `config`: Configuration management
//! - `ignore`: File ignore patterns handling
//! - `render`: Tree rendering logic
//! - `search`: Fuzzy search implementation
//! - `tree`: Directory tree data structure building
//! - `tui`: Terminal User Interface implementation

mod config;
mod ignore;
mod render;
mod search;
mod tree;
mod tui;

use std::io;
use std::path::PathBuf;
use std::process;

use clap::{CommandFactory, Parser};
use clap_complete::Shell;

use config::KreeConfig;
use ignore::IgnoreFilter;
use render::{build_color_map, build_icon_map, render_tree};
use search::{fuzzy_search, print_results};
use tree::{load_tree, SortMode};

/// Command Line Interface arguments parser for Kree.
#[derive(Parser)]
#[command(name = "kree", version, about = "A directory tree visualizer and fuzzy finder")]
struct Cli {
    /// Root directory to scan. Defaults to current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum depth to traverse the directory tree.
    #[arg(short, long)]
    depth: Option<u32>,

    /// Fuzzy search query to find a specific file or directory name.
    #[arg(short, long)]
    find: Option<String>,

    /// Show hidden files and ignore .kreeignore patterns.
    #[arg(short, long)]
    all: bool,

    /// Sort order for entries in the tree.
    #[arg(short, long, value_enum)]
    sort: Option<SortMode>,

    /// Disable colored output.
    #[arg(long)]
    no_color: bool,

    /// Show Nerd Font icons next to files and directories.
    #[arg(short = 'i', long)]
    icons: bool,

    /// Launch interactive TUI mode for navigation and preview.
    #[arg(short = 't', long)]
    tui: bool,

    /// Generate shell completion script and exit.
    #[arg(long, value_enum)]
    completions: Option<Shell>,
}

fn main() {
    let cli = Cli::parse();

    // Handle shell completions generation
    if let Some(shell) = cli.completions {
        clap_complete::generate(shell, &mut Cli::command(), "kree", &mut io::stdout());
        return;
    }

    // Load configuration from file (e.g., ~/.kreerc)
    let config = KreeConfig::load();

    // Merge CLI arguments with configuration defaults
    let depth = cli.depth.or(config.defaults.depth).unwrap_or(1);
    let sort = cli.sort.or(config.sort_mode()).unwrap_or(SortMode::Kind);
    let no_color = cli.no_color || config.defaults.no_color.unwrap_or(false);
    let icons = cli.icons || config.defaults.icons.unwrap_or(false);
    let all = cli.all || config.defaults.all.unwrap_or(false);

    // Configure colored output
    if no_color {
        colored::control::set_override(false);
    }

    // Safety check for depth to prevent stack overflow or excessive output
    if depth > 60 {
        println!("Depth overflow!!\nAre you serious?");
        process::exit(0);
    }

    // Run in TUI mode if requested
    if cli.tui {
        let filter = IgnoreFilter::new(!all, &config.ignore.patterns);
        let color_map = build_color_map(&config.colors);
        let icon_map = build_icon_map(&config.icons);
        let root = load_tree(&cli.path, depth, 0, &filter, sort);
        if let Err(e) = tui::run(root, cli.path.clone(), color_map, icon_map, filter, sort, depth) {
            eprintln!("TUI error: {e}");
            process::exit(1);
        }
    } 
    // Run fuzzy search if a query is provided
    else if let Some(query) = &cli.find {
        let results = fuzzy_search(&cli.path, query, depth);
        print_results(&results);
    } 
    // Standard tree rendering mode
    else {
        let filter = IgnoreFilter::new(!all, &config.ignore.patterns);
        let color_map = build_color_map(&config.colors);
        let icon_map = if icons { Some(build_icon_map(&config.icons)) } else { None };
        let root = load_tree(&cli.path, depth, 0, &filter, sort);
        render_tree(&root, &color_map, icon_map.as_ref());
    }
}
