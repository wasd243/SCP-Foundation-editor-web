use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    utils::check_diagnostic_without_include,
};

#[derive(Debug, Serialize)]
pub struct InlineBlocks<'lsp> {
    span: Span,
    content: Vec<&'lsp Token>,
}

impl<'lsp> InlineBlocks<'lsp> {
    pub fn from_parse_token_slice(
        tokens: &'lsp [Token],
        diagnostics: &'lsp [Diagnostic],
    ) -> Option<Self> {
        for token in tokens {
            if diagnostics
                .iter()
                .any(|d| check_diagnostic_without_include(token, d))
            {
                return None;
            }

            let start_mark = token;
            let mut content = Vec::new();
            let mut end_mark = None;

            for token in tokens {
                if token.ty == TokenTy::RightBlock {
                    end_mark = Some(token);
                    break;
                }
                content.push(token);
            }

            if let Some(end_mark) = end_mark {
                return Some(Self {
                    span: Span {
                        start: start_mark.span.start,
                        end: end_mark.span.end,
                    },
                    content,
                });
            }
        }
        None
    }

    pub fn content_len(&self) -> usize {
        self.content.len()
    }
}
