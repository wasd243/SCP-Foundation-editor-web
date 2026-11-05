use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    utils::check_diagnostic_without_include,
};

#[derive(Debug, Serialize)]
pub(super) struct InlineBlocks<'lsp> {
    span: Span,
    content: Vec<&'lsp Token>,
}

impl<'lsp> InlineBlocks<'lsp> {
    pub(super) fn from_parse_token_slice(
        tokens: &'lsp [Token],
        diagnostics: &'lsp [Diagnostic],
    ) -> Option<Self> {
        let (start_mark, rest) = tokens.split_first()?;
        let end_idx = rest.iter().position(|t| t.ty == TokenTy::RightBlock)?;
        let end_span = rest[end_idx].span.end;

        for token in tokens[..=end_idx + 1].iter() {
            if diagnostics
                .iter()
                .any(|d| check_diagnostic_without_include(token, d))
            {
                return None;
            }
        }

        let content = rest[..end_idx].iter().collect();

        Some(Self {
            span: Span {
                start: start_mark.span.start,
                end: end_span,
            },
            content,
        })
    }

    pub(super) fn content_len(&self) -> usize {
        self.content.len()
    }
}
