mod ignore;
mod render;
mod search;
mod tree;

use std::path::PathBuf;
use std::process;

use clap::Parser;

use ignore::IgnoreFilter;
use render::render_tree;
use search::{fuzzy_search, print_results};
use tree::load_tree;

#[derive(Parser)]
#[command(name = "kree", version, about = "A directory tree visualizer and fuzzy finder")]
struct Cli {
    /// Root directory to scan
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum depth to traverse
    #[arg(short, long, default_value_t = 1)]
    depth: u32,

    /// Fuzzy search for a file or directory name
    #[arg(short, long)]
    find: Option<String>,

    /// Show hidden files and ignore .kreeignore
    #[arg(short, long)]
    all: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.depth > 60 {
        println!("Depth overflow!!\nAre you serious?");
        process::exit(0);
    }

    if let Some(query) = &cli.find {
        let results = fuzzy_search(&cli.path, query, cli.depth);
        print_results(&results);
    } else {
        let filter = IgnoreFilter::new(!cli.all);
        let root = load_tree(&cli.path, cli.depth, 0, &filter);
        render_tree(&root);
    }
}
