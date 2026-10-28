## SCP-foundation-editor-web

A DSL code highlighter for my personal wasm-bindgen and AST practice.

### Build
```bash
cd ftml_handler
wasm-pack build --release --target bundler # build wasm pkg in the `ftml_handler/`
```
And then go back to the root folder use pnpm to build the frontend:
```bash
pnpm build # build frontend
```
