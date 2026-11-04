use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, ParseToken, Token},
    utils::has_diagnostic,
};

#[derive(Debug, Serialize)]
pub struct InlineMarks<'lsp> {
    pub span: Span,
    start_mark: &'lsp Token,
    content: Vec<&'lsp Token>,
    inner_marks: Vec<InlineMarks<'lsp>>,
    end_mark: &'lsp Token,
}

impl<'lsp> InlineMarks<'lsp> {
    pub fn from_parse_token(parse_token: &'lsp ParseToken) -> Option<Self> {
        let tokens = parse_token.tokens.as_slice();
        let diagnostics = parse_token.diagnostics.as_slice();
        from_parse_token_(tokens, diagnostics)
    }
}

fn from_parse_token_<'lsp>(
    tokens: &'lsp [Token],
    diagnostics: &[Diagnostic],
) -> Option<InlineMarks<'lsp>> {
    let mut iter = tokens.iter();

    while let Some(token) = iter.next() {
        if diagnostics.iter().any(|d| has_diagnostic(token, d)) {
            return None;
        }
        if !is_inline_mark_token(token.ty) {
            continue;
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
            } else if is_inline_mark_token(t.ty) {
                if let Some(nested) = from_parse_token_(rest, diagnostics) {
                    let n = nested.content.len();
                    content.extend(&rest[..n + 2]); // start + content + end of nested
                    iter.nth(n); // skip nested content + end mark; t is already consumed
                    inner_marks.push(nested);
                    continue;
                }
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

/// Check the token type for inlinemark.
fn is_inline_mark_token(ty: TokenTy) -> bool {
    matches!(
        ty,
        TokenTy::Bold
            | TokenTy::Italics
            | TokenTy::Underline
            | TokenTy::Superscript
            | TokenTy::Subscript
            | TokenTy::DoubleDash
            | TokenTy::DoubleTilde
    )
}
