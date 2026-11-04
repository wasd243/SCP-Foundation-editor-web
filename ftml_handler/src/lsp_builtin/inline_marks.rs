use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    utils::{has_diagnostic, is_inline_mark_token},
};

#[derive(Debug, Serialize)]
pub struct InlineMarks<'lsp> {
    span: Span,
    start_mark: &'lsp Token,
    content: Vec<&'lsp Token>,
    inner_marks: Vec<InlineMarks<'lsp>>,
    end_mark: &'lsp Token,
}

impl<'lsp> InlineMarks<'lsp> {
    pub(super) fn from_parse_token_slice(
        tokens_slice: &'lsp [Token],
        diagnostics: &'lsp [Diagnostic],
    ) -> Option<Self> {
        from_parse_token_(tokens_slice, diagnostics)
    }

    pub(super) fn content_len(&self) -> usize {
        self.content.len()
    }
}

/// This function is here for other Api args types.
fn from_parse_token_<'lsp>(
    tokens: &'lsp [Token],
    diagnostics: &'lsp [Diagnostic],
) -> Option<InlineMarks<'lsp>> {
    let mut iter = tokens.iter();

    while let Some(token) = iter.next() {
        if diagnostics.iter().any(|d| has_diagnostic(token, d)) {
            return None;
        }

        let start_mark = token;
        let mut content = Vec::new();
        let mut inner_marks = Vec::new();
        let mut end_mark = None;

        loop {
            let rest = iter.as_slice(); // taken before next(), so it starts at t
            let Some(t) = iter.next() else { break };

            if t.ty == start_mark.ty {
                end_mark = Some(t);
                break;
            } else if t.ty == TokenTy::RightMonospace && start_mark.ty == TokenTy::LeftMonospace {
                end_mark = Some(t);
                break;
            } else if is_inline_mark_token(t.ty)
                && let Some(nested) = from_parse_token_(rest, diagnostics)
            {
                let n = nested.content.len();
                content.extend(&rest[..n + 2]); // start + content + end of nested
                iter.nth(n); // skip nested content + end mark; t is already consumed
                inner_marks.push(nested);
                continue;
            }
            content.push(t);
        }

        if let Some(end_mark) = end_mark {
            return Some(InlineMarks {
                span: Span {
                    start: start_mark.span.start,
                    end: end_mark.span.end,
                },
                start_mark,
                content,
                inner_marks,
                end_mark,
            });
        }
    }

    None
}
