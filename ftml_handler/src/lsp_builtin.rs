mod inline_blocks;
mod inline_marks;

use inline_blocks::InlineBlocks;
use inline_marks::InlineMarks;

use crate::{
    ftml_ast::{Diagnostic, ParseToken, Token},
    utils::{is_inline_block_token, is_inline_mark_token},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum BuiltIn<'lsp> {
    InLineMarks(InlineMarks<'lsp>),
    InLineBlocks(InlineBlocks<'lsp>),
    Token(&'lsp Token),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct LspBuiltIn<'lsp> {
    tree: Vec<BuiltIn<'lsp>>,
    diagnostics: &'lsp Vec<Diagnostic>,
}

impl<'lsp> LspBuiltIn<'lsp> {
    pub fn visitor(parse_token: &'lsp ParseToken) -> Self {
        let mut tree = Vec::new();

        let tokens = &parse_token.tokens;
        let mut visitor_index = 0;
        while visitor_index < tokens.len() {
            if is_inline_mark_token(tokens[visitor_index].ty)
                && let Some(inline_mark) = InlineMarks::from_parse_token_slice(
                    &tokens[visitor_index..],
                    &parse_token.diagnostics,
                )
            {
                visitor_index += inline_mark.content_len() + 2; // which including start adn the end
                tree.push(BuiltIn::InLineMarks(inline_mark));
                continue;
            } else if is_inline_block_token(tokens[visitor_index].ty)
                && let Some(inline_block) = InlineBlocks::from_parse_token_slice(
                    &tokens[visitor_index..],
                    &parse_token.diagnostics,
                )
            {
                visitor_index += inline_block.content_len() + 2;
                tree.push(BuiltIn::InLineBlocks(inline_block));
                continue;
            } else {
                tree.push(BuiltIn::Token(&tokens[visitor_index]));
            }
            visitor_index += 1;
        }

        let diagnostics = &parse_token.diagnostics;

        Self { tree, diagnostics }
    }
}
