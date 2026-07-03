# rust_spreadsheet

A terminal spreadsheet engine written in Rust — vim-style navigation, cell formulas with dependency tracking, range functions, and full undo/redo, all built on a Cargo workspace with the core logic cleanly separated from the CLI.

## What it does

Type commands into a terminal REPL to build and navigate a spreadsheet:

- **Navigation**: `w`/`s`/`a`/`d` to scroll, `SCROLL_TO<CELL>` to jump (e.g. `SCROLL_TOB12`)
- **Formulas**: assign expressions to cells (`A1=5`, `B1=A1+2`), with automatic recalculation of dependents when a referenced cell changes
- **Range functions**: `SUM`, `AVG`, `MIN`, `MAX`, `STDEV` over cell ranges
- **Undo/redo**: `u` / `r`, backed by an operation stack that records old/new cell state for every edit
- **Output toggling**: `DISABLE_OUTPUT` / `ENABLE_OUTPUT` for benchmarking without terminal I/O overhead
- **Status line**: every command reports execution time and status (ok / invalid_cell / invalid_command / circular reference, etc.)

## Structure

This is a Cargo workspace with three members:

```
rust_spreadsheet/
├── spreadsheet_lib/    # core engine — no I/O, fully unit-testable
│   ├── sheet.rs        # command dispatch, formula evaluation, scrolling, display
│   ├── cell.rs         # cell value/formula representation
│   ├── dependency.rs   # dependency graph for recalculation
│   ├── region.rs       # cell ranges for SUM/AVG/etc.
│   ├── block.rs        # spreadsheet grid storage
│   └── undo.rs         # undo/redo operation stack
├── cli/                # thin stdin/stdout loop calling into spreadsheet_lib
├── server/             # planned web backend — currently just a stub
└── frontend/           # planned browser UI — currently empty placeholder files
```

The core design goal was decoupling: `spreadsheet_lib` has no knowledge of the terminal, stdin, or HTTP — it exposes `Sheet::execute_command()`, `display()`, and `print_status()`, so the same engine could drive a CLI, a server, or tests without changes.

**Honest status**: the CLI is complete and functional. `server/` and `frontend/` are early scaffolding for a planned browser-based version and don't do anything yet.

## Running it

```bash
cargo run -p cli
```

Type commands at the prompt (see above). `q` to quit.

## Tests

Core logic in `spreadsheet_lib` is unit-testable independent of the CLI:

```bash
cargo test -p spreadsheet_lib
```
