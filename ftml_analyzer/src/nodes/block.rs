//! # Block
//!
//! Block type including both `[[include :path:to:page]]` and `[[footnote]][[/footnote]]` types.
#[cfg(feature = "highlight")]
mod param;

#[cfg(feature = "highlight")]
pub use param::BlockParam;
#[cfg(feature = "highlight")]
use param::collect_params;

use serde::Serialize;

use crate::{Cursor, Span, TokenKind, mapping::Map};
#[cfg(feature = "highlight")]
use crate::{Token, highlight::AbsoluteToken};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum BlockKind {
    HasCloseTag,
    Inline,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Block {
    pub span: Span,
    pub ty: BlockKind,
    #[cfg(feature = "highlight")]
    pub params: Vec<BlockParam>,
    #[cfg(feature = "highlight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_param: Option<Token>,
    #[serde(skip)]
    len: usize,
}

/// Build and return a Block.
macro_rules! block {
    ($span_start:expr, $span_end:expr, $ty:expr, $params:ident, $end_param:expr, $len:expr) => {
        return Some($crate::nodes::block::Block {
            span: Span {
                start: $span_start.span.start,
                end: $span_end.span.end,
            },
            ty: $ty,
            #[cfg(feature = "highlight")]
            $params,
            #[cfg(feature = "highlight")]
            end_param: $end_param,
            len: $len,
        })
    };
}

impl Block {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn parse(cursor: &Cursor, map: &Map) -> Option<Self> {
        if cursor.token.ty != TokenKind::LeftBlock
            // in case `[[]]`
            || cursor.after.is_some_and(|t| t.ty == TokenKind::RightBlock)
        {
            return None;
        }

        let left_block = cursor.token;
        let rest = cursor.rest?;

        if let Some(open_end) = rest.iter().position(|t| t.ty == TokenKind::RightBlock) {
            let Some(block_end) = map.get(&cursor.index) else {
                let right_block = &rest[open_end];

                #[cfg(feature = "highlight")]
                let params = collect_params(left_block, cursor.after, &rest[..open_end]);

                block!(
                    left_block,
                    right_block,
                    BlockKind::Inline,
                    params,
                    // No end_param when block is written inline
                    None,
                    // add 3 to skip enough token.
                    open_end + Cursor::START_AND_END_MARK + 1
                );
            };

            let close_end = block_end - cursor.index - 2;
            let close = rest.get(close_end)?;

            debug_assert_eq!(close.ty, TokenKind::RightBlock);

            #[cfg(feature = "highlight")]
            let params = collect_params(left_block, cursor.after, &rest[..close_end]);

            #[cfg(feature = "highlight")]
            // [[/end]]
            //       ^^ close_end
            //    ^^^ close_end - 1
            let end_param = rest.get(close_end - 1)?;

            block!(
                left_block,
                close,
                BlockKind::HasCloseTag,
                params,
                Some(end_param.clone()),
                close_end
            );
        }

        None
    }

    #[cfg(feature = "highlight")]
    /// This function only convert params to AbsoluteTokens.
    pub fn to_abs_tokens(&self) -> Vec<AbsoluteToken> {
        use crate::highlight::HighlightKind;

        let mut tokens: Vec<AbsoluteToken> = self.params.iter().map(|p| p.to_abs_token()).collect();
        // Push the `[[` token into AbsoluteToken list.
        tokens.push(AbsoluteToken {
            line: self.span.start.0,
            start: self.span.start.1,
            len: 2,
            ty: HighlightKind::Operator,
            mods: 0,
        });

        // Push the `]]` token into AbsoluteToken list manually.
        if self.ty == BlockKind::Inline {
            tokens.push(AbsoluteToken {
                line: self.span.end.0,
                start: self.span.end.1.saturating_sub(2),
                len: 2,
                ty: HighlightKind::Operator,
                mods: 0,
            });
        } else if let Some(end_param) = &self.end_param {
            // Push the identifier on `[[/end]]` block into AbsoluteToken list manually.
            tokens.push(AbsoluteToken {
                line: end_param.span.start.0,
                start: end_param.span.start.1,
                len: end_param.span.end.1.saturating_sub(end_param.span.start.1),
                ty: HighlightKind::KeyWord,
                mods: 0,
            });
        }

        tokens
    }
}
