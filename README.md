# SCP-foundation-editor-web

A DSL code highlighter for my personal wasm-bindgen and AST practice.

## Build

```sh
cargo build --release # alright, you are done.
```

## Dependencies

This project use `lsp-server` and `lsp-types` implement the server.
The tokenization and diagnostic parts use the [ftml](https://github.com/scpwiki/ftml.git) crate.
