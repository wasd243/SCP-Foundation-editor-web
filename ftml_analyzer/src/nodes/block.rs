use serde::Serialize;

use crate::{Cursor, Span, nodes::CursorToNode};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum BlockKind {
    HasCloseTag,
    Inline,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct Block {
    pub span: Span,
    pub ty: BlockKind,
    #[serde(skip)]
    len: usize,
}

impl CursorToNode for Block {
    const COLLAPSIBLE: bool = true;
    impl_len!();
    fn parse(cursor: &Cursor) -> Option<Self> {
        if !cursor.is_in_block() {
            return None;
        }

        // let left_block_1 = cursor.token.span;
        unimplemented!()
    }
}
