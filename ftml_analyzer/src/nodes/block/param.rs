use serde::Serialize;

use crate::{Cursor, Span, Token, TokenKind};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct BlockParam {
    pub span: Span,
    pub ty: BlockParamKind,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[warn(clippy::arbitrary_source_item_ordering)]
#[rustfmt::skip]
pub enum BlockParamKind {
    Attr,    // css in `[[module css]]`
    KeyWord, // footnote in `[[footnote]]`
    String,  // for highlighting, mark a `string`
    Value,   // for |=value in `[[include]]` or other values in blocks 
}

macro_rules! param {
    ($span:expr, $ty:expr) => {
        return Some($crate::nodes::block::param::BlockParam {
            span: $span,
            ty: $ty,
        })
    };
}

impl BlockParam {
    pub fn parse_params(cursor: &Cursor) -> Option<Self> {
        if !cursor.is_in_block() {
            return None;
        }

        let current_span = cursor.token.span;

        if let Some(before) = cursor.before
            && cursor.token.ty == TokenKind::Identifier
        {
            // `[[module css]]`
            //          ^Whitespace
            //           ^^^Identifier -> Attr
            if before.ty == TokenKind::Whitespace {
                param!(current_span, BlockParamKind::Attr);
            }

            // `[[footnote]]`
            //  ^^LeftBlock
            //    ^^^^^^^^Identifier -> KeyWord
            if before.ty == TokenKind::LeftBlock {
                param!(current_span, BlockParamKind::KeyWord);
            }
        }

        // `"string"`
        // ^^^^^^^^^ -> String
        if cursor.is_string() {
            param!(current_span, BlockParamKind::String);
        }

        // `[[include :path:to:page
        // |var=value]]`
        //      ^^^^^ -> Value
        if cursor.is_value() {
            param!(current_span, BlockParamKind::Value);
        }

        None
    }
}

/// Build a sub cursor collect parameters.
pub(super) fn collect_params(inner: &[Token]) -> Vec<BlockParam> {
    let Some(mut sub) = Cursor::new(inner) else {
        return vec![];
    };

    let mut params = vec![];

    loop {
        if let Some(p) = BlockParam::parse_params(&sub) {
            params.push(p);
        }

        if sub.next().is_none() {
            break;
        }
    }
    params
}
