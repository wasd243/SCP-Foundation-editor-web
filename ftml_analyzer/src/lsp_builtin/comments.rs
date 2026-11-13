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
        if tokens
            .iter()
            .any(|t| diagnostics.iter().any(|d| has_diagnostic(t, d)))
        {
            return None;
        }

        let (start_token, rest) = tokens.split_first()?;

        let mut end_token = None;
        let mut contents = Vec::new();

        for token in rest {
            if token.ty == TokenTy::RightComment {
                end_token = Some(token);
                break;
            } else {
                contents.push(token);
            }
        }

        if let Some(end_token) = end_token {
            return Some(Comments {
                span: Span {
                    start: start_token.span.start,
                    end: end_token.span.end,
                },
                contents,
            });
        }

        None
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
