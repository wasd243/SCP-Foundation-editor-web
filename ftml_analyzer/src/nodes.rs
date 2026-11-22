/// A private mancro to generate `len()` implementations.
macro_rules! impl_len {
    () => {
        fn len(&self) -> usize {
            self.len
        }
    };
}

mod block;
mod comment;
#[cfg(feature = "highlight")]
mod inline_mark;

use block::Block;
use comment::Comment;
#[cfg(feature = "highlight")]
use inline_mark::InlineMark;

use serde::Serialize;

use crate::{
    Cursor, Span, Token,
    ftml_ast::{Diagnostic, ParseToken},
    mapping::build_map,
    nodes::block::BlockKind,
};

/// A trait for each node implementations.
trait CursorToNode {
    #[expect(unused)]
    /// Use `COLLAPSIBLE` to config LSP code collapse feature.
    const COLLAPSIBLE: bool;
    /// Try to parse at the cursor. Does not move the cursor on failure.
    fn parse(cursor: &Cursor) -> Option<Self>
    where
        Self: Sized;
    /// Output the private len field.
    fn len(&self) -> usize;
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
/// # Node
///
/// This enum represents individual nodes in the LSP builtin AST.
/// Each node has a struct with the same name.
/// This enum is constructed in `BuiltIn`.
pub enum Node {
    Block(Block),
    Comment(Comment),
    #[cfg(feature = "highlight")]
    InlineMark(InlineMark),
}

#[derive(Debug, Serialize)]
/// # BuiltIn
///
/// The LSP builtin AST. The `diagnostics` field is moved from `ParseToken`.
pub struct BuiltIn {
    pub nodes: Vec<Node>,
    pub diagnostics: Vec<Diagnostic>,
}

impl From<ParseToken> for BuiltIn {
    fn from(parse_token: ParseToken) -> Self {
        let nodes = Node::parse(parse_token.tokens);
        let diagnostics = parse_token.diagnostics;
        Self { nodes, diagnostics }
    }
}

impl Node {
    fn parse(tokens: Vec<Token>) -> Vec<Self> {
        let Some(mut cursor) = Cursor::new(&tokens) else {
            return vec![];
        };
        let map = build_map(&tokens);
        let mut nodes: Vec<Self> = vec![];

        // the main parse loop
        loop {
            if let Some(comment) = Comment::parse(&cursor) {
                cursor.nth_len(comment.len());
                nodes.push(Self::Comment(comment));
            }

            if let Some(block) = Block::parse(&cursor, &map) {
                if block.ty == BlockKind::Inline {
                    // nth_len: do not accept nested inline blocks
                    cursor.nth_len(block.len());
                }
                // no nth_len: step one token so nested blocks are visited
                nodes.push(Self::Block(block));
            }

            #[cfg(feature = "highlight")]
            if let Some(mark) = InlineMark::parse(&cursor) {
                nodes.push(Self::InlineMark(mark));
                // no nth_len: step one token so nested marks are visited
            }

            if cursor.next().is_none() {
                break;
            }
        }

        nodes
    }

    #[cfg(feature = "debug")]
    /// An info API for test harnnes (Debug only).
    pub fn info(&self) -> (&'static str, Span) {
        match self {
            Node::Block(b) => ("Block", b.span),
            Node::Comment(c) => ("Comment", c.span),
            #[cfg(feature = "highlight")]
            Node::InlineMark(m) => ("InlineMark", m.span),
        }
    }
}
