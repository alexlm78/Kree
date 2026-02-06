# Kree

A directory tree visualizer and fuzzy finder for the terminal, written in Rust.

## Features

- **Tree visualization** — Renders directory structures using Unicode box-drawing characters (`├──`, `└──`, `│`).
- **Fuzzy search** — Find files and directories by approximate name using Levenshtein distance.
- **Ignore rules** — Supports a `.kreeignore` file to exclude specific entries from the tree.
- **Hidden files** — Dot-prefixed files are hidden by default; use `-a` to reveal them.
- **Configurable depth** — Control how deep the tree traversal goes (max 60).

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

### Example output

```shell
.
├── Cargo.lock
├── Cargo.toml
├── CLAUDE.md
├── LICENSE
├── README.md
├── src
│    ├── ignore.rs
│    ├── main.rs
│    ├── render.rs
│    ├── search.rs
│    └── tree.rs
└── target
```

## `.kreeignore`

Create a `.kreeignore` file in your working directory to exclude entries from the tree. List one filename per line:

```shell
node_modules
target
dist
```

## License

MIT
