use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    lsp_highlight::{AbsoluteSpan, ToAbsoluteSpans},
    utils::has_diagnostic,
};

#[derive(Debug, Serialize)]
/// An special handleing for comments such as:
/// ```ftml, ignore
/// [!--
/// comments inside
/// --]
/// ```
pub struct Comments<'lsp> {
    pub span: Span,
    pub contents: Vec<&'lsp Token>,
}

impl<'lsp> Comments<'lsp> {
    pub fn from_parse_token_slice(
        tokens: &'lsp [Token],
        diagnostics: &[Diagnostic],
    ) -> Option<Self> {
        let start_token = tokens.first()?;

        // Unclosed comment: no RightComment found, fall back to None
        let end_idx = tokens.iter().position(|t| t.ty == TokenTy::RightComment)?;

        let slice = &tokens[..=end_idx];

        // Only check diagnostics inside the comment range, not the whole rest of the file
        if slice
            .iter()
            .any(|t| diagnostics.iter().any(|d| has_diagnostic(t, d)))
        {
            return None;
        }

        Some(Comments {
            span: Span {
                start: start_token.span.start,
                end: slice[end_idx].span.end,
            },
            contents: slice.iter().collect(),
        })
    }

    pub fn content_len(&self) -> usize {
        self.contents.len()
    }
}

impl ToAbsoluteSpans for Comments<'_> {
    fn to_abs_spans(&self) -> Vec<AbsoluteSpan> {
        self.contents
            .iter()
            .map(|t| t.to_comments_abs_span())
            .collect()
    }
}
