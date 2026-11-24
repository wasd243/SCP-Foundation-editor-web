use serde::Serialize;

#[cfg(feature = "highlight")]
use crate::highlight::{AbsoluteToken, HighlightKind};
use crate::{Cursor, Span, TokenKind, nodes::CursorToNode};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
/// # Comment
///
/// A comment in ftml, collapsible in LSP.
/// Usually written in:
/// ```ftml,ignore
/// [!--some comment inline--]
/// [!--
/// Also able to cross multiple lines
/// --]
/// ```
pub struct Comment {
    pub span: Span,
    #[serde(skip)]
    len: usize,
}

impl CursorToNode for Comment {
    const COLLAPSIBLE: bool = true;
    impl_len!();
    fn parse(cursor: &Cursor) -> Option<Self> {
        if cursor.token.ty != TokenKind::LeftComment {
            return None;
        }

        if let Some(after) = cursor.after
            && after.ty == TokenKind::RightComment
        {
            return Some(Self {
                span: Span {
                    start: cursor.token.span.start,
                    end: after.span.end,
                },
                len: Cursor::START_AND_END_MARK,
            });
        }

        let (i, end) = cursor
            .rest?
            .iter()
            .enumerate()
            .find(|(_, t)| t.ty == TokenKind::RightComment)?;

        Some(Self {
            span: Span {
                start: cursor.token.span.start,
                end: end.span.end,
            },

            // `rest` starts after `cursor.after`, so +1 for that token.
            // e.g. `[!--` `x` `--]` is 3 tokens, with `--]` at rest[0].
            len: i + Cursor::START_AND_END_MARK + 1,
        })
    }
}

#[cfg(feature = "highlight")]
impl Comment {
    pub fn to_abs_tokens(self) -> Vec<AbsoluteToken> {
        let (start_li, start_col) = self.span.start;
        let (end_li, end_col) = self.span.end;

        if start_li == end_li {
            return vec![AbsoluteToken {
                line: start_li,
                start: start_col,
                len: end_col - start_col,
                ty: HighlightKind::Comment,
                mods: 0,
            }];
        }

        let mut abs_tokens: Vec<AbsoluteToken> =
            Vec::with_capacity((end_li - start_li + 1) as usize);
        for line in start_li..=end_li {
            let start = if line != start_li { 0 } else { start_col };

            // HACK: Use u32::MAX to reach the end of line... Because I
            // don't want put that into `Comment` struct fields...
            let len = if line == end_li { end_col } else { u32::MAX };

            let abs_token = AbsoluteToken {
                line,
                start,
                len,
                ty: HighlightKind::Comment,
                mods: 0,
            };

            abs_tokens.push(abs_token);
        }

        abs_tokens
    }
}
