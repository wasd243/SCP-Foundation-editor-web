use serde::Serialize;

use crate::{Cursor, Span, TokenKind};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
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

impl Comment {
    /// Use `COLLAPSIBLE` to config LSP code collapse feature.
    pub const COLLAPSIBLE: bool = true;

    /// Try to parse one comment at the cursor. Does not move the cursor on failure.
    pub fn parse(cursor: &Cursor) -> Option<Self> {
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

    pub fn len(&self) -> usize {
        self.len
    }
}
