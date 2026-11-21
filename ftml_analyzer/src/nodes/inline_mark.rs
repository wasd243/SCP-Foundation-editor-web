//! # InlineMark
//!
//! This node contains all inline marks grammar such as:
//! ```ftml,ignore
//! **bold** //italic// __underline__ --corssthrough-- @@raw@@ etc.
//! ```
//!
//! Due to other extensions' support, this feature is not turned on by default.
use crate::{Cursor, Span, Token, TokenKind, nodes::CursorToNode};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct InlineMark {
    pub span: Span,
    pub ty: InlineMarkKind,
    #[serde(skip)]
    len: usize,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum InlineMarkKind {
    Bold,
    Italic,
    Underline,
    StrikeThrough,
    Monospace,
    Raw,
    Superscript,
    Subscript,
}

impl From<TokenKind> for InlineMarkKind {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Bold => Self::Bold,
            TokenKind::Italics => Self::Italic,
            TokenKind::Underline => Self::Underline,
            TokenKind::DoubleDash => Self::StrikeThrough,
            TokenKind::LeftMonospace => Self::Monospace,
            TokenKind::Raw => Self::Raw,
            TokenKind::Superscript => Self::Superscript,
            TokenKind::Subscript => Self::Subscript,
            _ => unreachable!(
                "Should not use this function when `TokenKind` is not inline mark token"
            ),
        }
    }
}

impl CursorToNode for InlineMark {
    const COLLAPSIBLE: bool = false;

    fn parse(cursor: &Cursor) -> Option<Self> {
        if !is_inline_mark_token_(cursor.token.ty)
            && !matches!(cursor.token.ty, TokenKind::LeftMonospace)
        {
            return None;
        }

        let ty = InlineMarkKind::from(cursor.token.ty);

        if let Some(after) = cursor.after
            && (is_inline_mark_token(after.ty, ty) || is_monospace(after.ty, ty))
            && !cursor.token.is_cross_line_with(after)
        {
            return Some(Self {
                span: Span {
                    start: cursor.token.span.start,
                    end: after.span.end,
                },
                ty,
                len: Cursor::START_AND_END_MARK,
            });
        }

        let (i, end) = find_close_mark(cursor, ty)?;

        // No cross line inline mark token
        if cursor.token.is_cross_line_with(end) {
            return None;
        }

        Some(Self {
            span: Span {
                start: cursor.token.span.start,
                end: end.span.end,
            },
            ty,
            len: i + Cursor::START_AND_END_MARK + 1,
        })
    }

    fn len(&self) -> usize {
        self.len
    }
}

fn find_close_mark<'c>(cursor: &Cursor<'c>, ty: InlineMarkKind) -> Option<(usize, &'c Token)> {
    let mut i = 0;
    let rest = cursor.rest?;
    while i < rest.len() {
        let t = &rest[i];

        if is_inline_mark_token(t.ty, ty) || is_monospace(t.ty, ty) {
            return Some((i, t)); // found the end
        }

        if (is_inline_mark_token_(t.ty) || t.ty == TokenKind::LeftMonospace)
            && let Some(sub) = Cursor::new(&rest[i..])
            && let Some(inner) = InlineMark::parse(&sub)
        {
            // nested mark: need its length to skip it
            // build a sub-cursor starting at rest[i..], parse, then skip
            i += inner.len();
            continue;
        }

        i += 1;
    }
    None
}

/// A helper function to check inline mark token, except monospace.
fn is_inline_mark_token(token_kind: TokenKind, previous_mark_ty: InlineMarkKind) -> bool {
    is_inline_mark_token_(token_kind) && InlineMarkKind::from(token_kind) == previous_mark_ty
}

/// A helper function to check inline token marks.
fn is_monospace(token_kind: TokenKind, previous_mark_ty: InlineMarkKind) -> bool {
    matches!(token_kind, TokenKind::RightMonospace) && previous_mark_ty == InlineMarkKind::Monospace
}

/// Inner function for inline token parsing.
fn is_inline_mark_token_(token_kind: TokenKind) -> bool {
    matches!(
        token_kind,
        TokenKind::Bold
            | TokenKind::Italics
            | TokenKind::Underline
            | TokenKind::DoubleDash
            | TokenKind::Raw
            | TokenKind::Superscript
            | TokenKind::Subscript
    )
}
