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
mod export;
mod ignore;
mod render;
mod search;
mod tree;
mod tui;

use std::io;
use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;

use config::KreeConfig;
use export::{export_json, export_markdown, export_yaml};
use ignore::IgnoreFilter;
use render::{build_color_map, build_icon_map, render_tree};
use search::{content_search, fuzzy_search, print_content_results, print_results};
use tree::{SortMode, TreeOptions, count_max_depth, load_tree};

/// Output format for tree export.
#[derive(Clone, ValueEnum)]
enum ExportFormat {
    Json,
    Yaml,
    Markdown,
}

/// Command Line Interface arguments parser for Kree.
#[derive(Parser)]
#[command(
    name = "kree",
    version,
    about = "A directory tree visualizer and fuzzy finder"
)]
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

    /// Search for a string inside file contents (grep-like).
    #[arg(short, long)]
    grep: Option<String>,

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

    /// Show only directories, hiding all files.
    #[arg(long)]
    dirs_only: bool,

    /// Filter by file extensions (comma-separated, e.g. rs,toml,md).
    /// Directories are always shown to preserve tree structure.
    #[arg(short = 'e', long, value_delimiter = ',')]
    extensions: Vec<String>,

    /// Show file metadata (size, permissions, date, owner).
    #[arg(short = 'l', long)]
    long: bool,

    /// Export tree in a specific format instead of rendering.
    #[arg(short = 'F', long, value_enum)]
    format: Option<ExportFormat>,

    /// Disable .gitignore rules (by default, .gitignore is respected).
    #[arg(long)]
    no_gitignore: bool,

    /// Generate shell completion script and exit.
    #[arg(long, value_enum)]
    completions: Option<Shell>,

    /// Generate man page and print to stdout.
    #[arg(long)]
    man: bool,

    /// Print the maximum directory depth and exit.
    /// Only counts directories as levels. Useful to discover how deep
    /// the tree goes before rendering with `-d`.
    #[arg(short = 'L', long, conflicts_with_all = ["find", "grep", "tui", "format"])]
    levels: bool,
}

fn main() {
    let cli = Cli::parse();

    // Handle shell completions generation
    if let Some(shell) = cli.completions {
        clap_complete::generate(shell, &mut Cli::command(), "kree", &mut io::stdout());
        return;
    }

    // Handle man page generation
    if cli.man {
        let cmd = Cli::command();
        let man = clap_mangen::Man::new(cmd);
        man.render(&mut io::stdout())
            .expect("Failed to generate man page");
        return;
    }

    // Load configuration from file (e.g., ~/.kreerc)
    let config = KreeConfig::load();

    // Merge CLI arguments with configuration defaults
    let depth = if cli.levels && cli.depth.is_none() {
        // When counting levels without an explicit depth cap, scan fully (up to safety limit)
        60
    } else {
        cli.depth.or(config.defaults.depth).unwrap_or_else(|| {
            // Smart default depth: expand more levels for small directories
            let count = fs::read_dir(&cli.path)
                .map(|entries| entries.filter_map(|e| e.ok()).count())
                .unwrap_or(0);
            if count <= 10 {
                3
            } else if count <= 30 {
                2
            } else {
                1
            }
        })
    };
    let sort = cli.sort.or(config.sort_mode()).unwrap_or(SortMode::Kind);
    let no_color = cli.no_color || config.defaults.no_color.unwrap_or(false);
    let icons = cli.icons || config.defaults.icons.unwrap_or(false);
    let all = cli.all || config.defaults.all.unwrap_or(false);
    let use_gitignore = !cli.no_gitignore && !config.defaults.no_gitignore.unwrap_or(false);
    let opts = TreeOptions {
        dirs_only: cli.dirs_only,
        extensions: cli
            .extensions
            .iter()
            .map(|e| e.trim_start_matches('.').to_lowercase())
            .collect(),
        show_metadata: cli.long,
    };

    // Configure colored output
    if no_color {
        colored::control::set_override(false);
    }

    // Safety check for depth to prevent stack overflow or excessive output
    if depth > 60 {
        println!("Depth overflow!!\nAre you serious?");
        process::exit(0);
    }

    // Print maximum directory depth and exit
    if cli.levels {
        let filter =
            IgnoreFilter::with_gitignore(!all, &config.ignore.patterns, use_gitignore, &cli.path);
        let max = count_max_depth(&cli.path, depth, 0, &filter, &opts);
        println!("{max}");
        return;
    }

    // Run in TUI mode if requested
    if cli.tui {
        let filter =
            IgnoreFilter::with_gitignore(!all, &config.ignore.patterns, use_gitignore, &cli.path);
        let color_map = build_color_map(&config.colors);
        let icon_map = build_icon_map(&config.icons);
        let root = load_tree(&cli.path, depth, 0, &filter, sort, &opts);
        if let Err(e) = tui::run(
            root,
            cli.path.clone(),
            color_map,
            icon_map,
            filter,
            sort,
            depth,
            opts,
        ) {
            eprintln!("TUI error: {e}");
            process::exit(1);
        }
    }
    // Run content search if --grep is provided
    else if let Some(query) = &cli.grep {
        let results = content_search(&cli.path, query, depth);
        print_content_results(&results);
    }
    // Run fuzzy search if a query is provided
    else if let Some(query) = &cli.find {
        let results = fuzzy_search(&cli.path, query, depth);
        print_results(&results);
    }
    // Export mode
    else if let Some(format) = &cli.format {
        let filter =
            IgnoreFilter::with_gitignore(!all, &config.ignore.patterns, use_gitignore, &cli.path);
        let root = load_tree(&cli.path, depth, 0, &filter, sort, &opts);
        let output = match format {
            ExportFormat::Json => export_json(&root),
            ExportFormat::Yaml => export_yaml(&root),
            ExportFormat::Markdown => export_markdown(&root),
        };
        print!("{output}");
    }
    // Standard tree rendering mode
    else {
        let filter =
            IgnoreFilter::with_gitignore(!all, &config.ignore.patterns, use_gitignore, &cli.path);
        let color_map = build_color_map(&config.colors);
        let icon_map = if icons {
            Some(build_icon_map(&config.icons))
        } else {
            None
        };
        let root = load_tree(&cli.path, depth, 0, &filter, sort, &opts);
        render_tree(&root, &color_map, icon_map.as_ref());
    }
}
