//! # Block
//!
//! Block type including both `[[include :path:to:page]]` and `[[footnote]][[/footnote]]` types.
#[cfg(feature = "highlight")]
mod param;

#[cfg(feature = "highlight")]
pub use param::BlockParam;
use param::collect_params;

use serde::Serialize;

use crate::{Cursor, Span, TokenKind, mapping::Map};

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
    #[serde(skip)]
    len: usize,
}

/// Build and return a Block.
macro_rules! block {
    ($span_start:expr, $span_end:expr, $ty:expr, $params:ident,  $len:ident) => {
        return Some($crate::nodes::block::Block {
            span: Span {
                start: $span_start.span.start,
                end: $span_end.span.end,
            },
            ty: $ty,
            #[cfg(feature = "highlight")]
            $params,
            len: $len,
        })
    };
}

impl Block {
    #[expect(unused)]
    const COLLAPSIBLE: bool = true;
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

        if let Some(i1) = rest.iter().position(|t| t.ty == TokenKind::RightBlock) {
            let Some(block_end) = map.get(&cursor.index) else {
                let right_block = &rest[i1];

                #[cfg(feature = "highlight")]
                let params = collect_params(&rest[..=i1]);

                block!(left_block, right_block, BlockKind::Inline, params, i1);
            };

            let i2 = block_end - cursor.index - Cursor::START_AND_END_MARK;
            let close = &rest[i2];

            #[cfg(feature = "highlight")]
            let params = collect_params(&rest[..=i2]);

            block!(left_block, close, BlockKind::HasCloseTag, params, i2);
        }

        None
    }
}
