# Copilot Instructions for SCP Foundation Editor (Web)

## Build, lint, and test commands

- Install dependencies: `npm ci` (CI/default) or `npm install` (local)
- Build parser + bundle: `npm run build`
  - Runs `npx lezer-generator src/wikidot.grammar -o src/parser.js` and then `node build.js`
- Lint all JS: `npm run lint`
- Lint a single file: `npx eslint editor.js` (replace path as needed)
- Serve built site locally: `npm run serve` (port `3000`)
- Build + serve in one step: `npm start`
- Tests: `npm test` is currently a placeholder that exits with error (`"no test specified"`), so there is no single-test command configured yet.

## High-level architecture

The app is a browser-hosted CodeMirror 6 editor customized for Wikidot syntax and usually embedded into Wikidot pages via Tampermonkey.

1. **Grammar/parser layer**
   - Source grammar is `src/wikidot.grammar`, with custom tokenizer rules in `src/token.js`.
   - `src/parser.js` and `src/parser.terms.js` are generated outputs consumed by the editor runtime.

2. **Editor runtime**
   - Entry point is `editor.js`.
   - It wires a custom Lezer language, highlight tags/classes, fold rules, autocomplete (`component/completion.js`), and color tooling (`component/color_widgets.js`, `component/color_preview.js`).
   - Mixed-language parsing is enabled for `[[module css]]...[[/module]]` and `[[html]]...[[/html]]` blocks.

3. **Presentation layer**
   - `index.html` hosts the editor container and top-level UI controls (sync/init/clear + quick formatting toolbar).
   - `assets/code_view.css` defines most `cm-*` token class styles used by `editor.js`.

4. **Wikidot integration**
   - `userscript/h2o2-wikidot-editor.user.js` injects the editor as a right-side iframe panel into Wikidot edit pages.
   - Cross-window sync uses `postMessage` with `h2o2-init` (Wikidot ➜ iframe) and `h2o2-update` (iframe ➜ Wikidot textarea).

5. **Build/deploy**
   - `build.js` bundles to `assets/bundle_editor.js` and writes `assets/build-info.json`.
   - `.github/workflows/deploy.yml` regenerates parser + builds + deploys to GitHub Pages.

## Key conventions for this repository

- **Do not hand-edit generated parser files.** Keep edits in `src/wikidot.grammar` and `src/token.js`, then regenerate `src/parser.js`.
- **Syntax feature changes are cross-file by design:** when adding/changing a token, update at least:
  1. Grammar/tokenizer (`src/wikidot.grammar`, `src/token.js`)
  2. Tag mapping/highlighting/folding (`editor.js`)
  3. CSS token classes (`assets/code_view.css`)
- **Wikidot bridge contract is stable and explicit:** preserve `h2o2-init`/`h2o2-update` message types and origin checks when touching iframe sync.
- **Userscript deployment assumption:** default `EDITOR_URL` points to the GitHub Pages build; if deployment target changes, update userscript `EDITOR_URL`.
- **Color syntax support has two paths:** Wikidot color blocks (`###RRGGBB|text##`) and generic hex color previews (`#RGB`, `#RRGGBB`, `#RRGGBBAA`) are handled by different extensions.
- **Autocomplete is intentionally snippet-heavy and SCP-CN specific:** `component/completion.js` contains many hardcoded Wikidot/component templates and cursor placement logic via `view.dispatch`.
