# Kree

A directory tree visualizer and fuzzy finder for the terminal, written in Rust.

## Features

- **Tree visualization** — Renders directory structures using Unicode box-drawing characters (`├──`, `└──`, `│`).
- **Fuzzy search** — Find files and directories by approximate name using Levenshtein distance.
- **Ignore rules** — Supports a `.kreeignore` file to exclude specific entries from the tree.
- **Hidden files** — Dot-prefixed files are hidden by default; use `-a` to reveal them.
- **Configurable depth** — Control how deep the tree traversal goes (max 60).
- **Sort modes** — Sort alphabetically (`-s name`) or group directories first (`-s kind`).
- **Colored output** — Directories in blue, executables in green, regular files in bright white.
- **Nerd Font icons** — Opt-in file-type icons via `--icons` (requires a [Nerd Font](https://www.nerdfonts.com/)).
- **Configuration file** — Persistent defaults, custom colors, icons, and global ignore patterns via `~/.kreerc`.

## Installation

```shell
cargo build --release
```

The binary will be at `target/release/Kree`.

## Usage

```shell
# Show current directory tree (depth 1 by default)
kree

# Show tree of a specific directory with depth 3
kree /some/path -d 3

# Fuzzy search for a file or directory
kree -f main

# Show hidden files and disable .kreeignore filtering
kree -a

# Sort by kind (directories first)
kree -s kind

# Show Nerd Font icons (requires a Nerd Font)
kree --icons

# Combine options
kree /some/path -d 5 -f config
```

### Options

| Flag | Long      | Description                                | Default |
|------|-----------|--------------------------------------------|---------|
|      | `[PATH]`  | Root directory to scan                     | `.`     |
| `-d` | `--depth` | Maximum depth to traverse                  | `1`     |
| `-f` | `--find`  | Fuzzy search for a file or directory name  |         |
| `-a` | `--all`   | Show hidden files and ignore `.kreeignore` | `false` |
| `-s` | `--sort`  | Sort order: `name` or `kind`               | `kind`  |
| `-i` | `--icons` | Show Nerd Font icons next to entries        | `false` |

### Example output

```shell
└── .
     ├── src
     │    ├── ignore.rs
     │    ├── main.rs
     │    ├── render.rs
     │    ├── search.rs
     │    └── tree.rs
     ├── target
     ├── Cargo.lock
     ├── Cargo.toml
     ├── LICENSE
     └── README.md
```

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

| Section      | Key        | Type       | Description                                         |
|--------------|------------|------------|-----------------------------------------------------|
| `[defaults]` | `depth`    | integer    | Default traversal depth (overridden by `-d`)        |
| `[defaults]` | `sort`     | string     | `"name"` or `"kind"` (overridden by `-s`)           |
| `[defaults]` | `no_color` | boolean    | Disable colors (overridden by `--no-color`)         |
| `[defaults]` | `all`      | boolean    | Show hidden files (overridden by `-a`)              |
| `[defaults]` | `icons`    | boolean    | Show Nerd Font icons (overridden by `-i`)           |
| `[colors]`   | `<ext>`    | string     | Color for file extension — named color or hex code  |
| `[icons]`    | `<ext>`    | string     | Icon for file extension — any Unicode character     |
| `[ignore]`   | `patterns` | string[]   | Filenames to always exclude (merged with `.kreeignore`) |

### Supported colors

Named ANSI colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`.

Hex truecolor: any `#RRGGBB` value (e.g. `#FF6600`).

Hyphens and underscores are interchangeable in color names (e.g. `bright-red` and `bright_red` both work).

## `.kreeignore`

Create a `.kreeignore` file in your working directory to exclude entries from the tree. List one filename per line:

```shell
node_modules
target
dist
```

## Acknowledgments

This project is inspired by [Dree](https://github.com/ujjwall-R/Dree) by [@ujjwall-R](https://github.com/ujjwall-R), a terminal-based file exploration tool written in C++. Kree is a reimplementation in Rust with its own approach to tree rendering, fuzzy search, and colored output.

## License

MIT
