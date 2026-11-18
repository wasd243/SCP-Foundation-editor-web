vim.filetype.add({ extension = { ftml = "ftml" } })

-- setup the soft wrap
vim.api.nvim_create_autocmd("FileType", {
	group = vim.api.nvim_create_augroup("FtmlSoftWrap", { clear = true }),
	pattern = "ftml",
	callback = function()
		vim.opt_local.wrap = true
		vim.opt_local.linebreak = true
		vim.opt_local.breakindent = true
		vim.keymap.set({ "n", "v" }, "j", "gj", { buffer = true })
		vim.keymap.set({ "n", "v" }, "k", "gk", { buffer = true })

		vim.schedule(function()
			-- Override global pair mappings for this buffer with plain keys.
			vim.keymap.set("i", "[", "[", { buffer = true })
			vim.keymap.set("i", "]", "]", { buffer = true })
		end)
	end,
})

-- setup LSP custom highlights.
local function apply()
	local set = vim.api.nvim_set_hl
	set(0, "@lsp.type.ftmlBold.ftml", { bold = true })
	set(0, "@lsp.type.ftmlItalic.ftml", { italic = true })
	set(0, "@lsp.type.ftmlUnderline.ftml", { underline = true })
	set(0, "@lsp.type.ftmlStrike.ftml", { strikethrough = true })
	set(0, "@lsp.type.ftmlMonospace.ftml", { link = "@markup.raw" })
	set(0, "@lsp.type.ftmlLinkUrl.ftml", { link = "@markup.link.url" })
	-- super/sub scripts are unset due to terminal rendering issues
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
			cmd = { vim.fn.expand("~/project/SCP-Foundation-editor-web/target/release/ftml_analyzer") },
			filetypes = { "ftml" },
			root_markers = { ".git" },
		})

		return vim.tbl_deep_extend("force", opts or {}, {
			servers = {
				ftml_analyzer = {},
			},
		})
	end,
}
