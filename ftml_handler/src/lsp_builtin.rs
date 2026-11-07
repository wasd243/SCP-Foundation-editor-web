mod blocks;
mod inline_blocks;
mod inline_marks;
mod regex;

use blocks::Blocks;
use inline_blocks::InlineBlocks;
use inline_marks::InlineMarks;

use crate::{
    ftml_ast::{Diagnostic, ParseToken, Token},
    utils::{is_inline_block_token, is_inline_mark_token},
};

use ftml::parsing::Token as TokenTy;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum BuiltIn<'lsp> {
    InLineMarks(InlineMarks<'lsp>),
    InLineBlocks(InlineBlocks<'lsp>),
    Blocks(Blocks<'lsp>),
    Token(&'lsp Token),
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BlockKind {
    #[default]
    Block,
    ModuleCss,
    Html,
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
            let (node, used) = visit(&tokens[visitor_index..], &parse_token.diagnostics);
            tree.push(node);
            visitor_index += used;
        }

        Self {
            tree,
            diagnostics: &parse_token.diagnostics,
        }
    }
}

fn visit<'lsp>(
    parse_token: &'lsp [Token],
    diagnostics: &'lsp [Diagnostic],
) -> (BuiltIn<'lsp>, usize) {
    if is_inline_mark_token(parse_token[0].ty)
        && let Some(inline_marks) = InlineMarks::from_parse_token_slice(parse_token, diagnostics)
    {
        let used = inline_marks.content_len() + 2;
        return (BuiltIn::InLineMarks(inline_marks), used);
    } else if is_inline_block_token(parse_token[0].ty)
        && let Some(inline_blocks) = InlineBlocks::from_parse_token_slice(parse_token, diagnostics)
    {
        let used = inline_blocks.content_len() + 2;
        return (BuiltIn::InLineBlocks(inline_blocks), used);
    } else if parse_token[0].ty == TokenTy::LeftBlock
        && let Some(bloks) = Blocks::from_parse_token(parse_token, diagnostics)
    {
        let used = bloks.token_len();
        return (BuiltIn::Blocks(bloks), used);
    }
    (BuiltIn::Token(&parse_token[0]), 1)
}
