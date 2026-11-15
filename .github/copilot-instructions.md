# Copilot instructions

## Project shape

This is a Rust workspace for an LSP that parses FTML and publishes editor features for `.ftml` files. The workspace has two crates:

- `ftml_analyzer` contains the FTML tokenizer, built-in AST visitor, diagnostics, folding ranges, semantic-token conversion, and the embedded web-language service.
- `lsp_driver` is the small executable entrypoint that starts `ftml_analyzer::build_server()` over stdio. The workspace default member is `lsp_driver`.

FTML parsing is layered: `ftml_analyzer/src/ftml_ast.rs` produces parser tokens, `src/lsp_builtin/` groups them into an editor-oriented tree, and `src/lsp_highlight.rs` plus `src/utils/highlight.rs` converts that tree into absolute spans and then LSP semantic-token deltas. `src/server.rs` owns the stdio loop, document buffers, notifications, diagnostics, folding requests, and semantic-token requests.

HTML and CSS are embedded-language services rather than Rust parsers. `ftml_analyzer/src/lsp_builtin/types/visual_document.rs` identifies embedded regions, while the Deno/TypeScript code under `ftml_analyzer/deno/src/` calls `vscode-html-languageservice` and `vscode-css-languageservice` for virtual documents. `ftml_analyzer/build.rs` bundles `deno/src/main.ts` into `OUT_DIR/bundle.js`; `src/server/web_service.rs` executes that bundle inside `deno_core`.

## Build, test, and lint

The Deno CLI must be installed before building `ftml_analyzer`, because its Cargo build script invokes `deno bundle`.

From the repository root:

```sh
# Build the default LSP executable
cargo build --release

# Run all analyzer tests
cargo test -p ftml_analyzer

# Run the CI-equivalent lint
cargo clippy -p ftml_analyzer --all-targets -- -D warnings
```

The README's release-build path is also valid:

```sh
cd lsp_driver
cargo build --release
```

Run one integration test by test name, or one test module by target:

```sh
cargo test -p ftml_analyzer --test token check_builtin
cargo test -p ftml_analyzer --test lsp check_highlight_abs_token
```

The snapshot-style tests in `ftml_analyzer/tests/token.rs` and `tests/lsp.rs` support regeneration with the `bless` feature. Use this only when intentionally updating expected JSON:

```sh
cargo test -p ftml_analyzer --features bless --test token
cargo test -p ftml_analyzer --features bless --test lsp
```

CI runs Clippy and tests with `working-directory: ./ftml_analyzer`, so the equivalent commands from that directory are `cargo clippy --all-targets -- -D warnings` and `cargo test`.

## Repository conventions

- Normalize CRLF to LF before parsing. Use the existing `parse_token`/`set_up_parsing!` path instead of bypassing normalization.
- Keep parser spans and semantic-token coordinates consistent with the source after whitespace sanitization. Semantic tokens must be sorted and encoded as LSP relative deltas; preserve the legend in `lsp_highlight/types.rs` when adding token kinds.
- `Span.start` and `Span.end` line/column values already use the editor/LSP coordinate convention. Pass them through directly; do not recalculate columns from byte offsets or apply another coordinate conversion when combining embedded results.
- Only edit files explicitly marked by the user. If a change appears to require touching an unmarked dependent file, stop and request permission instead of widening the change set.
- Before implementing new conversion, parsing, or request-handling logic, check whether the active dependency or existing code already provides a suitable API. Reuse a known API when one exists; do not add redundant custom implementations.
- Do not make unnecessary changes. Extend and reuse existing functions and call paths instead of bypassing or duplicating them.
- Before performing any potentially risky operation or introducing a potentially risky implementation, explain the risk and obtain the user's permission. Do not silently proceed with known risks or present an unapproved risky implementation as complete.
- If the user explicitly prohibits tests or validation commands for a task, do not run them; document the restriction rather than overriding it.
- Treat `LspBuiltIn` as an editor/highlighting tree, not the full FTML semantic model. Add grouping logic in the appropriate `lsp_builtin` submodule and keep token-level parsing in `ftml_ast`.
- The server uses full-document synchronization. Changes replace the entire buffer, and diagnostics are republished after open/change and cleared on close.
- Embedded-language requests should go through the existing Deno bundle and `WebService` boundary. Keep the TypeScript request shape (`lang`, `method`, `text`, `position`) aligned with the Rust caller and return explicit JSON errors.
- Snapshot fixtures under `ftml_analyzer/tests/{tokens,builtin,abs_tokens}` are checked byte-for-byte. Update fixtures deliberately with `--features bless`; do not hand-edit generated output.
- Custom FTML semantic token types are mapped in `nvim_plugin.lua`; keep new custom legend entries and editor highlight links synchronized.
- The project is still incomplete: syntax completion and HTML/CSS highlighting are active work areas. Do not document or assume complete embedded-language support; add focused coverage when extending those paths.
