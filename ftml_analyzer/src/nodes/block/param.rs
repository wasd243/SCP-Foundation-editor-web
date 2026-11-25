use serde::Serialize;

use crate::{
    Cursor, Span, Token, TokenKind,
    highlight::{AbsoluteToken, HighlightKind},
};

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
    Equals,  // =
    KeyWord, // footnote in `[[footnote]]`
    Pipe,    // |
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
            && matches!(
                cursor.token.ty,
                // special handling for `[[=]]` `[[>]]` `[[<]]`
                TokenKind::Identifier | TokenKind::Other | TokenKind::Quote | TokenKind::Equals
            )
        {
            // `[[module css]]`
            //          ^Whitespace
            //           ^^^Identifier -> Attr
            if before.ty == TokenKind::Whitespace || before.ty == TokenKind::Pipe {
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
        if cursor.is_value() && !matches!(cursor.token.ty, TokenKind::Whitespace | TokenKind::Pipe)
        {
            param!(current_span, BlockParamKind::Value);
        }

        match cursor.token.ty {
            TokenKind::Pipe => param!(current_span, BlockParamKind::Pipe),
            TokenKind::Equals => param!(current_span, BlockParamKind::Equals),
            _ => {}
        }

        None
    }

    pub(super) fn to_abs_token(self) -> AbsoluteToken {
        let ty = match self.ty {
            BlockParamKind::Attr => HighlightKind::Attr,
            BlockParamKind::Equals => HighlightKind::Equals,
            BlockParamKind::KeyWord => HighlightKind::KeyWord,
            BlockParamKind::String => HighlightKind::String,
            BlockParamKind::Value => HighlightKind::Value,
            BlockParamKind::Pipe => HighlightKind::Pipe,
        };

        AbsoluteToken {
            line: self.span.start.0,
            start: self.span.start.1,
            // NOTE: This will led to overflow if any token crosses any line.
            len: self.span.end.1 - self.span.start.1,
            ty,
            mods: 0,
        }
    }
}

/// Build a sub cursor collect parameters.
pub(super) fn collect_params(
    left_block: &Token,
    keyword: Option<&Token>,
    inner: &[Token],
) -> Vec<BlockParam> {
    let mut params = vec![];

    if let Some(keyword) = keyword
        && let Some(sub) = Cursor::new_in_block(std::slice::from_ref(keyword), left_block)
        && let Some(param) = BlockParam::parse_params(&sub)
    {
        params.push(param);
    }

    let before = keyword.unwrap_or(left_block);
    let Some(mut sub) = Cursor::new_in_block(inner, before) else {
        return params;
    };

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
