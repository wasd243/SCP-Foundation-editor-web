# FTML Analyzer Web Extension

The package runs the FTML language server in a Web Worker backed by WebAssembly.

## Build

Install the `wasm32-unknown-unknown` Rust target and `wasm-bindgen-cli` version
0.2.130, then run these commands from this directory:

```sh
pnpm install
pnpm run build
```

The build script runs `cargo build -p ftml_lsp_wasm --target
wasm32-unknown-unknown --release --locked`, generates the web-target
wasm-bindgen bindings in `wasm/`, then bundles the extension and worker in
`dist/`.

To package a VSIX after building, run:

```sh
pnpm exec vsce package
```
