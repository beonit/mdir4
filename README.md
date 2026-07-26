# Mdir4

Mdir4 is a Rust terminal file manager that recreates the keyboard-driven,
multi-column navigation experience of Mdir III.

## Available features

- Real directory listing with a synthetic parent `..` entry
- Directories-first, ascending name sort
- One to six adaptive, column-major columns
- Spatial Up/Down/Left/Right navigation
- Home/End and Page Up/Page Down
- Enter to open a directory and Backspace to go to its parent
- Space/Insert/Ctrl+A marking
- R to refresh
- F1 help
- Unicode filename truncation based on terminal cell width
- Safe warning below the minimum 60×15 terminal size
- Ctrl+Q confirmation to quit and restore the terminal

Rename, View, Edit, Copy, Move, Delete, MCD, QCD, and Menu are shown but are not
implemented yet. Pressing Enter on a regular file launches its platform default application.

## Run

Start in the current directory:

```sh
cargo run --locked
```

Start in a specified directory:

```sh
cargo run --locked -- /path/to/directory
```

Build and run a release binary:

```sh
cargo build --release --locked
./target/release/mdir4 /path/to/directory
```

## Keys

| Key | Action |
|---|---|
| `Up / Down` | Move vertically; cross pages at the first/last visible item |
| `Left / Right` | Move between columns and adjacent pages |
| `Enter` | Open directory or launch a regular file with the platform default application |
| `Backspace` | Go to parent directory |
| `Home / End` | Select first/last item |
| `PgUp / PgDn` | Select previous/next page |
| `Space` | Toggle mark on current item |
| `Insert` | Toggle mark and move down |
| `Ctrl+A` | Mark all markable items |
| `R` | Refresh |
| `F1` | Open help |
| `Esc` | Close help or clear a message |
| `Ctrl+Q` | Open quit confirmation |

## Verify

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
```

See the [`documentation map`](docs/README.md) for document priority, the active milestone,
and detailed implementation plans.

Post-v1 plans:

- [`Git built-in`](docs/plugins/git/README.md)
- [`SSH Remote / Remote Drive`](docs/remote/README.md)
