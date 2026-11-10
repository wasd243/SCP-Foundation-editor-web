use ftml::parsing::Token as TokenTy;
use serde::{Serialize, Serializer};

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    lsp_highlight::{AbsoluteSpan, HighlightTy, ToAbsoluteSpans},
    utils::{has_diagnostic, is_inline_mark_token},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct InlineMarks<'lsp> {
    pub span: Span,
    #[serde(skip)]
    content: Vec<&'lsp Token>,
    pub inner_marks: Vec<InlineMarks<'lsp>>,
    #[serde(serialize_with = "serialize_ty")]
    pub ty: TokenTy,
}

impl<'lsp> InlineMarks<'lsp> {
    #[must_use]
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

            if t.ty == start_mark.ty
                || (t.ty == TokenTy::RightMonospace && start_mark.ty == TokenTy::LeftMonospace)
                || (t.ty == TokenTy::RightComment && start_mark.ty == TokenTy::LeftComment)
            {
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
            let ty = end_mark.ty;
            return Some(InlineMarks {
                span: Span {
                    start: start_mark.span.start,
                    end: end_mark.span.end,
                },
                content,
                inner_marks,
                ty,
            });
        }
    }

    None
}

/// Convert `RightMonospace` or `LeftMonospace` to `monospace`.
fn serialize_ty<S>(ty: &TokenTy, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match ty {
        TokenTy::LeftMonospace | TokenTy::RightMonospace => serializer.serialize_str("monospace"),
        TokenTy::LeftComment | TokenTy::RightComment => serializer.serialize_str("comment"),
        other => other.serialize(serializer),
    }
}

impl ToAbsoluteSpans for InlineMarks<'_> {
    fn to_abs_spans(&self) -> Vec<AbsoluteSpan> {
        if self.inner_marks.is_empty() {
            vec![to_abs_span_(self)]
        } else {
            self.inner_marks
                .iter()
                .flat_map(|m| m.to_abs_spans())
                .collect()
        }
    }
}

/// To Absolute Span Inner.
fn to_abs_span_(inline_marks: &InlineMarks) -> AbsoluteSpan {
    AbsoluteSpan {
        line: inline_marks.span.start.0,
        start: inline_marks.span.start.1,
        len: inline_marks.span.end.1 - inline_marks.span.start.1,
        ty: match_inline_marks_ty(inline_marks.ty),
        // TODO: mods is a placeholder
        mods: 0,
    }
}

fn match_inline_marks_ty(ty: TokenTy) -> HighlightTy {
    match ty {
        TokenTy::Bold => HighlightTy::Bold,
        TokenTy::Italics => HighlightTy::Italics,
        TokenTy::Underline => HighlightTy::Underline,
        TokenTy::DoubleDash => HighlightTy::StrikeThrough,
        TokenTy::Superscript => HighlightTy::Superscript,
        TokenTy::Subscript => HighlightTy::Subscript,
        TokenTy::Raw => HighlightTy::Raw,
        TokenTy::LeftComment | TokenTy::RightComment => HighlightTy::Comment,
        TokenTy::LeftMonospace | TokenTy::RightMonospace => HighlightTy::Monospace,
        _ => unreachable!("Should never reach other token type in `inline_marks`"),
    }
}
