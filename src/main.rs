mod config;
mod ignore;
mod render;
mod search;
mod tree;

use std::path::PathBuf;
use std::process;

use clap::Parser;

use config::KreeConfig;
use ignore::IgnoreFilter;
use render::{build_color_map, render_tree};
use search::{fuzzy_search, print_results};
use tree::{load_tree, SortMode};

#[derive(Parser)]
#[command(name = "kree", version, about = "A directory tree visualizer and fuzzy finder")]
struct Cli {
    /// Root directory to scan
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum depth to traverse
    #[arg(short, long)]
    depth: Option<u32>,

    /// Fuzzy search for a file or directory name
    #[arg(short, long)]
    find: Option<String>,

    /// Show hidden files and ignore .kreeignore
    #[arg(short, long)]
    all: bool,

    /// Sort order for entries
    #[arg(short, long, value_enum)]
    sort: Option<SortMode>,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,
}

fn main() {
    let cli = Cli::parse();
    let config = KreeConfig::load();

    let depth = cli.depth.or(config.defaults.depth).unwrap_or(1);
    let sort = cli.sort.or(config.sort_mode()).unwrap_or(SortMode::Kind);
    let no_color = cli.no_color || config.defaults.no_color.unwrap_or(false);
    let all = cli.all || config.defaults.all.unwrap_or(false);

    if no_color {
        colored::control::set_override(false);
    }

    if depth > 60 {
        println!("Depth overflow!!\nAre you serious?");
        process::exit(0);
    }

    if let Some(query) = &cli.find {
        let results = fuzzy_search(&cli.path, query, depth);
        print_results(&results);
    } else {
        let filter = IgnoreFilter::new(!all, &config.ignore.patterns);
        let color_map = build_color_map(&config.colors);
        let root = load_tree(&cli.path, depth, 0, &filter, sort);
        render_tree(&root, &color_map);
    }
}
