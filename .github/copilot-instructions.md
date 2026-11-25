# Copilot instructions

## Build, test, and lint

Run commands from the workspace root:

```sh
# Build the LSP executable
cargo build --release -p lsp_driver

# Build with semantic highlighting enabled
cargo build --release -p lsp_driver --features highlight

# Run all workspace tests (including the snapshot suites)
cargo test --workspace

# Run analyzer unit tests or one named unit test
cargo test -p ftml_analyzer
cargo test -p ftml_analyzer line_col_at_eof

# Run the block snapshot test group
cargo test -p tests --test block

# Run the analyzer Clippy command used by CI
cargo clippy -p ftml_analyzer --all-targets -- -D warnings
```

Snapshot cases live under `tests/tests/`. To intentionally regenerate their `builtin.json` files, run `cargo test -p tests --features bless`.

## Architecture

This Rust workspace implements an FTML language server. `ftml_analyzer` parses source and builds editor-facing structure; `ftml_lsp` exposes that functionality over the Language Server Protocol; `lsp_driver` starts the server over stdio. `lsp_test` and the `tests` crate provide the snapshot/expectation test harness.

Parsing flows through `ftml_analyzer::get_ast`: `ftml_ast::ParseToken` uses the upstream `ftml` crate to tokenize and parse warnings, then converts token and diagnostic spans; `nodes::BuiltIn` groups those tokens into editor-oriented block and comment nodes (and inline-mark nodes with highlighting enabled). `Cursor` carries token context during grouping, while `mapping::build_map` pairs block delimiters. `BuiltIn` is for editor features, not a complete FTML semantic model.

`ftml_lsp` owns document buffers and dispatches notifications and requests. It uses full-document synchronization; open/change updates the buffer and republishes parser diagnostics, while close removes the buffer and clears diagnostics. Folding and completion are registered in the current server; semantic-token support is conditional on the `highlight` feature.

## Repository conventions

- Use `get_ast` and the existing `ParseToken` path when parsing source. It applies the parser's token cleanup and diagnostic filtering before node grouping.
- `Span` positions are converted from UTF-8 byte offsets to UTF-16 line/column positions in `ftml_ast/span.rs`, matching LSP coordinates. Preserve these coordinates when passing spans to LSP responses; avoid recalculating columns from byte offsets.
- Keep token extraction in `ftml_ast` and editor-node grouping in `nodes/`; use the existing `Cursor` and block map rather than duplicating their context/matching logic.
- Semantic highlighting is opt-in. When changing token kinds, keep `HighlightKind` conversion and the ordered legend in `ftml_analyzer/src/highlight/` aligned; custom FTML token names also have Neovim highlight links in `nvim_plugin.lua`. Semantic tokens are sorted and encoded as LSP relative deltas in `nodes.rs`.
- Snapshot test groups (`block`, `comment`, `inline_mark`) read case directories containing `input.ftml`, generated `builtin.json`, and optional `expect.toml`. Update generated JSON through the `bless` feature, not by hand.
