use ftml::parsing::Token as TokenTy;
use serde::{Serialize, Serializer};

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    lsp_builtin::{BuiltIn, visit},
    lsp_highlight::{AbsoluteSpan, HighlightTy, ToAbsoluteSpans},
    utils::{has_diagnostic, highlight::color_nodes},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct InlineMarks<'lsp> {
    pub span: Span,
    #[serde(skip)]
    content: Vec<&'lsp Token>,
    pub inner: Vec<BuiltIn<'lsp>>,
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

    while let Some(start_mark) = iter.next() {
        if diagnostics.iter().any(|d| has_diagnostic(start_mark, d)) {
            return None;
        }

        let mut content = Vec::new();
        let mut inner = Vec::new();
        let mut end_mark = None;

        loop {
            let rest = iter.as_slice(); // taken before next(), so it starts at t
            let Some(t) = iter.next() else { break };

            if t.ty == start_mark.ty
                || (t.ty == TokenTy::RightMonospace && start_mark.ty == TokenTy::LeftMonospace)
            {
                end_mark = Some(t);
                break;
            }

            if matches!(start_mark.ty, TokenTy::Raw) {
                content.push(t);
                continue;
            }

            let (node, used) = visit(rest, diagnostics);
            content.extend(&rest[..used]);
            for _ in 1..used {
                iter.next(); // t is already consumed
            }

            // normal tokens won't get into `inner`, inner only contains nodes
            if !matches!(node, BuiltIn::Token(_)) {
                inner.push(node);
            }
        }

        if let Some(end_mark) = end_mark {
            let ty = end_mark.ty;
            return Some(InlineMarks {
                span: Span {
                    start: start_mark.span.start,
                    end: end_mark.span.end,
                },
                content,
                inner,
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
        other => other.serialize(serializer),
    }
}

impl ToAbsoluteSpans for InlineMarks<'_> {
    fn to_abs_spans(&self) -> Vec<AbsoluteSpan> {
        let mut abs_spans = Vec::new();

        if self.inner.is_empty() {
            abs_spans.push(to_abs_span_(self));
        } else {
            for node in &self.inner {
                color_nodes(node, &mut abs_spans);
            }
        }

        abs_spans
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
        TokenTy::LeftMonospace | TokenTy::RightMonospace => HighlightTy::Monospace,
        _ => unreachable!("Should never reach other token type in `inline_marks`"),
    }
}
