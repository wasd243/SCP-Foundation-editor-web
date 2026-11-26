# ftml analyzer

An LSP (Language Server Protocol) server for FTML (Foundation Text Markup Language). It provides completions, folding ranges, and diagnostics in your editor.

Syntax highlighting is available as an **optional** feature, intended for editors that don't have a community highlighting plugin.

Available under the terms of the GNU Affero General Public License. See [LICENSE](LICENSE).

> [!NOTE]
> `[[html]]` and `[[module CSS]]` blocks contain embedded languages. Completions and diagnostics are not available inside them.

## Building

Two cargo aliases are provided (see `.cargo/config.toml`):

*These aliases will expand `--release` automatically.*

```bash
cargo buildlsp        # build without the highlighting feature
cargo buildhighlight  # build with the highlighting feature
```

## IDE plugins

### VS Code

<!--Place Holder Before publish extension on VS Code extension market-->

### Neovim

Requires Neovim 0.11+ and [lazy.nvim](https://github.com/folke/lazy.nvim) (the default in LazyVim).

1. Build the server (see [Building](#building)).
2. Copy [nvim_plugin.lua](extensions/nvim_plugin.lua) to `~/.config/nvim/lua/plugins/ftml_analyzer.lua`.
3. Set `cmd` in that file to the path of your built binary. There is a comment in the file marking the line to change.
4. Restart Neovim and open a `.ftml` file.

```lua
vim.lsp.config("ftml_analyzer", {
  -- use your build directory like this.
  cmd = { vim.fn.expand("path to your LSP") },
  filetypes = { "ftml" },
  root_markers = { ".git" },
})
```

## Dependencies

This repo uses `lsp-server` and `lsp-types` to implement the server. The tokenization and diagnostic parts use the [ftml](https://github.com/scpwiki/ftml.git) crate.
