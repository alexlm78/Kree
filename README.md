# Kree

A directory tree visualizer and fuzzy finder for the terminal, written in Rust.

## Project Structure

This repository is managed as a Cargo Workspace:

- **`kree_cli/`** — The core terminal tool (CLI).
- **`www/`** — Official website built with Leptos (WASM).
- **`docs/`** — Architecture, design specifications, and manuals.

## Features

- **Tree visualization** — Renders directory structures using Unicode box-drawing characters (`├──`, `└──`, `│`).
- **Fuzzy search** — Find files and directories by approximate name using Levenshtein distance.
- **Content search** — Search inside file contents with grep-like output (`--grep`).
- **Ignore rules** — Supports `.kreeignore` (glob patterns) and `.gitignore` (auto-detected).
- **Hidden files** — Dot-prefixed files are hidden by default; use `-a` to reveal them.
- **Smart depth** — Automatically expands more levels for small directories when no explicit depth is given.
- **File metadata** — Show permissions, owner, size, and modification date (`--long`).
- **Export formats** — Output tree as JSON, YAML, or Markdown (`--format`).
- **Sort modes** — Sort alphabetically (`-s name`) or group directories first (`-s kind`).
- **Colored output** — Directories in blue, executables in green, 40+ extensions with custom colors.
- **Nerd Font icons** — Opt-in file-type icons via `--icons` (requires a [Nerd Font](https://www.nerdfonts.com/)).
- **Interactive TUI** — Full-screen interactive mode with keyboard navigation, expand/collapse, lazy loading, file preview, and inline fuzzy search (`--tui`).
- **Configuration file** — Persistent defaults, custom colors, icons, and global ignore patterns via `~/.kreerc`.
- **Shell completions** — Auto-generated for bash, zsh, fish, powershell, and elvish.
- **Man page** — Generate with `kree --man`.
- **Parallel traversal** — Uses rayon for fast scanning of large directory trees.

## Installation

### From crates.io

```shell
cargo install kree
```

### Homebrew (macOS)

```shell
brew tap alexlm78/kree
brew install kree
```

### From Source (Workspace)

```shell
cargo build --release -p kree
```

The binary will be at `target/release/kree`.

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/alexlm78/Kree/releases) — available for Linux (x64), Windows (x64), and macOS (arm64 + x64).

## Usage

```shell
# Show current directory tree (smart depth auto-detected)
kree

# Show tree of a specific directory with depth 3
kree /some/path -d 3

# Fuzzy search for a file or directory
kree -f main

# Search inside file contents (grep-like)
kree -g "TODO" -d 5

# Show file metadata (permissions, owner, size, date)
kree -l

# Export tree as JSON
kree -F json -d 3

# Export tree as Markdown
kree -F markdown > tree.md

# Show hidden files and disable .kreeignore filtering
kree -a

# Disable .gitignore rules
kree --no-gitignore

# Filter by extension
kree -e rs,toml

# Show only directories
kree --dirs-only

# Show Nerd Font icons
kree --icons

# Launch interactive TUI mode
kree -t

# Generate man page
kree --man > kree.1

# Combine options
kree /some/path -d 5 -l --icons -s kind
```

### Options

| Flag | Long             | Description                                  | Default    |
|------|------------------|----------------------------------------------|------------|
|      | `[PATH]`         | Root directory to scan                       | `.`        |
| `-d` | `--depth`        | Maximum depth to traverse                    | smart auto |
| `-f` | `--find`         | Fuzzy search for a file or directory name    |            |
| `-g` | `--grep`         | Search inside file contents                  |            |
| `-a` | `--all`          | Show hidden files and ignore `.kreeignore`   | `false`    |
| `-s` | `--sort`         | Sort order: `name` or `kind`                 | `kind`     |
| `-l` | `--long`         | Show file metadata (permissions, size, date) | `false`    |
| `-F` | `--format`       | Export format: `json`, `yaml`, `markdown`    |            |
| `-e` | `--extensions`   | Filter by extensions (comma-separated)       |            |
| `-i` | `--icons`        | Show Nerd Font icons next to entries         | `false`    |
| `-t` | `--tui`          | Launch interactive TUI mode                  | `false`    |
|      | `--dirs-only`    | Show only directories, hide files            | `false`    |
|      | `--no-color`     | Disable colored output                       | `false`    |
|      | `--no-gitignore` | Disable `.gitignore` rules                   | `false`    |
|      | `--completions`  | Generate shell completion script and exit    |            |
|      | `--man`          | Generate man page and print to stdout        |            |

### Example output

```shell
└── .
     ├── src
     │    ├── config.rs
     │    ├── export.rs
     │    ├── ignore.rs
     │    ├── main.rs
     │    ├── render.rs
     │    ├── search.rs
     │    ├── tree.rs
     │    └── tui.rs
     ├── Cargo.lock
     ├── Cargo.toml
     ├── LICENSE
     └── README.md

1 directories, 11 files
```

With `--long`:

```
└── .  rwxr-xr-x  user   544B  Mar 14 2026 01:30
     ├── src  rwxr-xr-x  user    96B  Feb  9 2026 04:51
     ├── Cargo.toml  rw-r--r--  user   133B  Mar  3 2026 15:24
     └── README.md  rw-r--r--  user   7.7K  Mar  3 2026 15:52
```

### TUI keybindings

| Key                     | Mode   | Action                         |
|-------------------------|--------|--------------------------------|
| `q` / `Esc`             | Normal | Quit                           |
| `Ctrl+C`                | Any    | Quit                           |
| `Up` / `k`              | Normal | Cursor up                      |
| `Down` / `j`            | Normal | Cursor down                    |
| `Right` / `l` / `Enter` | Normal | Expand directory (lazy loads)  |
| `Left` / `h`            | Normal | Collapse dir or jump to parent |
| `/`                     | Normal | Enter search mode              |
| `Home` / `End`          | Normal | Jump to first/last entry       |
| `PageUp` / `PageDown`   | Normal | Scroll by half viewport        |
| `r`                     | Normal | Reload tree from disk          |
| Any char                | Search | Append to query (live filter)  |
| `Backspace`             | Search | Delete last char from query    |
| `Enter`                 | Search | Confirm search, back to Normal |
| `Esc`                   | Search | Cancel search, clear query     |
| `Up` / `Down`           | Search | Jump to prev/next match        |

## Configuration (`~/.kreerc`)

Kree supports a TOML configuration file at `~/.kreerc` for setting persistent defaults, custom extension colors, and global ignore patterns. CLI arguments always override config values.

### Sample config

A full example is available at [`kreerc.example`](kreerc.example) in the root of this repository. Copy it to get started:

```shell
cp kreerc.example ~/.kreerc
```

```toml
# ~/.kreerc — Kree configuration file
# CLI arguments always override these values.

[defaults]
depth = 3              # default traversal depth
sort = "kind"          # "name" or "kind"
no_color = false       # disable colored output
all = false            # show hidden files
icons = false          # show Nerd Font icons
no_gitignore = false   # disable .gitignore support

[colors]
# Named ANSI colors or hex truecolor (#RRGGBB)
rs = "#FF6600"         # Rust — orange
py = "green"           # Python
go = "cyan"            # Go
js = "yellow"          # JavaScript
java = "bright_red"    # Java
log = "bright_black"   # Log files — dimmed

[icons]
# Override icons per extension or special key (directory, executable, default)
# rs = ""

[ignore]
# Merged with local .kreeignore; -a flag overrides both
patterns = ["target", "node_modules", "dist", "__pycache__", ".git"]
```

### Sections

| Section      | Key            | Type       | Description                                         |
|--------------|----------------|------------|-----------------------------------------------------|
| `[defaults]` | `depth`        | integer    | Default traversal depth (overridden by `-d`)        |
| `[defaults]` | `sort`         | string     | `"name"` or `"kind"` (overridden by `-s`)           |
| `[defaults]` | `no_color`     | boolean    | Disable colors (overridden by `--no-color`)         |
| `[defaults]` | `all`          | boolean    | Show hidden files (overridden by `-a`)              |
| `[defaults]` | `icons`        | boolean    | Show Nerd Font icons (overridden by `-i`)           |
| `[defaults]` | `no_gitignore` | boolean    | Disable .gitignore support (overridden by `--no-gitignore`) |
| `[colors]`   | `<ext>`        | string     | Color for file extension — named color or hex code  |
| `[icons]`    | `<ext>`        | string     | Icon for file extension — any Unicode character     |
| `[ignore]`   | `patterns`     | string[]   | Filenames to always exclude (merged with `.kreeignore`) |

### Supported colors

Named ANSI colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`.

Hex truecolor: any `#RRGGBB` value (e.g. `#FF6600`).

Hyphens and underscores are interchangeable in color names (e.g. `bright-red` and `bright_red` both work).

## Shell Completions

Generate autocompletion scripts for your shell:

```shell
# Bash
kree --completions bash >> ~/.bashrc

# Zsh
kree --completions zsh > ~/.zfunc/_kree

# Fish
kree --completions fish > ~/.config/fish/completions/kree.fish
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

## `.kreeignore`

Create a `.kreeignore` file in your working directory to exclude entries from the tree. Supports full glob syntax:

```shell
# Exact names
node_modules
target

# Glob patterns
*.log
build_*
**/*.tmp
```

## Acknowledgments

This project is inspired by [Dree](https://github.com/ujjwall-R/Dree) by [@ujjwall-R](https://github.com/ujjwall-R), a terminal-based file exploration tool written in C++. Kree is a reimplementation in Rust with its own approach to tree rendering, fuzzy search, and colored output.

## License

MIT
