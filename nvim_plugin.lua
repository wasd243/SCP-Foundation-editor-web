vim.filetype.add({ extension = { ftml = "ftml" } })

-- setup LSP custom highlights.
local function apply()
  local set = vim.api.nvim_set_hl
  set(0, "@lsp.type.ftmlBold.ftml", { bold = true })
  set(0, "@lsp.type.ftmlItalic.ftml", { italic = true })
  set(0, "@lsp.type.ftmlUnderline.ftml", { underline = true })
  set(0, "@lsp.type.ftmlStrike.ftml", { strikethrough = true })
  set(0, "@lsp.type.ftmlMonospace.ftml", { link = "@markup.raw" })
end

vim.api.nvim_create_autocmd("ColorScheme", {
  group = vim.api.nvim_create_augroup("FtmlLspHighlights", { clear = true }),
  callback = apply,
})

apply()

return {
  "neovim/nvim-lspconfig",
  opts = function(_, opts)
    vim.lsp.config("ftml_analyzer", {
      -- use your build directory like this.
      cmd = { vim.fn.expand("~/SCP-Foundation-editor-web/lsp_driver/target/release/ftml_analyzer_lsp") },
      filetypes = { "ftml" },
      root_markers = { ".git" }
    })

    return vim.tbl_deep_extend("force", opts or {}, {
      servers = {
        ftml_analyzer = {},
      },
    })
  end
}
