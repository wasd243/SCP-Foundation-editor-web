use serde::Serialize;

#[cfg(feature = "highlight")]
use crate::highlight::{AbsoluteToken, HighlightKind};
use crate::{Cursor, Span, TokenKind, nodes::CursorToNode};

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
    #[cfg(feature = "highlight")]
    #[serde(skip)]
    line_ends: Vec<(u32, u32)>,
}

impl CursorToNode for Comment {
    impl_len!();
    fn parse(cursor: &Cursor) -> Option<Self> {
        if cursor.token.ty != TokenKind::LeftComment {
            return None;
        }

        if let Some(after) = cursor.after
            && after.ty == TokenKind::RightComment
        {
            #[cfg(feature = "highlight")]
            let line_ends = if cursor.token.span.start.0 == after.span.end.0 {
                Vec::new()
            } else {
                comment_line_ends(std::iter::once(cursor.token).chain(std::iter::once(after)))
            };

            return Some(Self {
                span: Span {
                    start: cursor.token.span.start,
                    end: after.span.end,
                },
                len: Cursor::START_AND_END_MARK,
                #[cfg(feature = "highlight")]
                line_ends,
            });
        }

        let rest = cursor.rest?;
        let (i, end) = rest
            .iter()
            .enumerate()
            .find(|(_, t)| t.ty == TokenKind::RightComment)?;

        #[cfg(feature = "highlight")]
        let line_ends = if cursor.token.span.start.0 == end.span.end.0 {
            Vec::new()
        } else {
            comment_line_ends(
                std::iter::once(cursor.token)
                    .chain(cursor.after)
                    .chain(rest[..=i].iter()),
            )
        };

        Some(Self {
            span: Span {
                start: cursor.token.span.start,
                end: end.span.end,
            },

            // `rest` starts after `cursor.after`, so +1 for that token.
            // e.g. `[!--` `x` `--]` is 3 tokens, with `--]` at rest[0].
            len: i + Cursor::START_AND_END_MARK + 1,
            #[cfg(feature = "highlight")]
            line_ends,
        })
    }
}

#[cfg(feature = "highlight")]
fn comment_line_ends<'a>(tokens: impl Iterator<Item = &'a crate::Token>) -> Vec<(u32, u32)> {
    let mut line_ends: Vec<(u32, u32)> = Vec::new();
    let mut push_end = |line: u32, end_col: u32| {
        if let Some((last_line, last_end)) = line_ends.last_mut()
            && *last_line == line
        {
            *last_end = (*last_end).max(end_col);
        } else {
            line_ends.push((line, end_col));
        }
    };

    for token in tokens {
        let (start_line, start_col) = token.span.start;
        let (end_line, end_col) = token.span.end;
        if start_line == end_line {
            push_end(end_line, end_col);
            continue;
        }

        let mut line = start_line;
        let mut col = start_col;
        let mut parts = token.slice.split('\n').peekable();
        while let Some(part) = parts.next() {
            let has_next_line = parts.peek().is_some();
            let part = part.strip_suffix('\r').unwrap_or(part);
            let part_end = if has_next_line || line != end_line {
                col + part.encode_utf16().count() as u32
            } else {
                end_col
            };
            push_end(line, part_end);

            if has_next_line {
                line += 1;
                col = 0;
            }
        }
    }

    line_ends
}

#[cfg(feature = "highlight")]
impl Comment {
    pub fn to_abs_tokens(&self) -> Vec<AbsoluteToken> {
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

        let mut abs_tokens = Vec::with_capacity(self.line_ends.len());
        for &(line, end_col) in &self.line_ends {
            let start = if line == start_li { start_col } else { 0 };
            let len = end_col.saturating_sub(start);
            if len == 0 {
                continue;
            }

            abs_tokens.push(AbsoluteToken {
                line,
                start,
                len,
                ty: HighlightKind::Comment,
                mods: 0,
            });
        }

        abs_tokens
    }
}

#[cfg(all(test, feature = "highlight"))]
mod tests {
    use crate::{get_ast, highlight::HighlightKind, nodes::Node};

    const FTML: &str = "[!--\n\n--]\n[!--\ncontent\n\n--]\n[[div]]";

    #[test]
    fn multiline_comment_highlighting_stops_at_comment_end() {
        let ast = get_ast(FTML);
        let abs_tokens: Vec<_> = ast
            .nodes
            .iter()
            .flat_map(|node| match node {
                Node::Comment(comment) => comment.to_abs_tokens(),
                _ => Vec::new(),
            })
            .collect();
        let spans: Vec<_> = abs_tokens
            .iter()
            .map(|token| (token.line, token.start, token.len, token.ty.to_index()))
            .collect();

        assert_eq!(
            spans,
            [
                (0, 0, 4, HighlightKind::Comment.to_index()),
                (2, 0, 3, HighlightKind::Comment.to_index()),
                (3, 0, 4, HighlightKind::Comment.to_index()),
                (4, 0, 7, HighlightKind::Comment.to_index()),
                (6, 0, 3, HighlightKind::Comment.to_index()),
            ]
        );
        assert!(ast.nodes.iter().any(|node| matches!(node, Node::Block(_))));
    }
}
