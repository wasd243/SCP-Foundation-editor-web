mod inline_marks;

use inline_marks::InlineMarks;

use crate::{
    ftml_ast::{Diagnostic, ParseToken},
    utils::is_inline_mark_token,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
enum BuiltIn<'lsp> {
    InLineMarks(InlineMarks<'lsp>),
}

#[derive(Debug, Serialize)]
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
                && let Some(mark) = InlineMarks::from_parse_token_slice(
                    &parse_token.tokens[visitor_index..],
                    &parse_token.diagnostics,
                )
            {
                visitor_index += mark.content_len() + 2; // which including start adn the end
                tree.push(BuiltIn::InLineMarks(mark));
                continue;
            }
            visitor_index += 1;
        }

        let diagnostics = &parse_token.diagnostics;

        Self { tree, diagnostics }
    }
}
