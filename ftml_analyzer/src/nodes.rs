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

#[cfg(feature = "highlight")]
use lsp_types::SemanticToken;
use serde::Serialize;

#[cfg(feature = "debug")]
use crate::Span;
#[cfg(feature = "highlight")]
use crate::highlight::color_nodes;
use crate::{
    Cursor, Token,
    ftml_ast::{Diagnostic, ParseToken},
    mapping::build_map,
    nodes::block::BlockKind,
};

/// A trait for each node implementations.
trait CursorToNode {
    /// Try to parse at the cursor. Does not move the cursor on failure.
    fn parse(cursor: &Cursor) -> Option<Self>
    where
        Self: Sized;
    /// Output the private len field.
    fn len(&self) -> usize;
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[warn(clippy::arbitrary_source_item_ordering)]
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
    #[cfg(any(feature = "highlight", feature = "debug"))]
    Token(Token),
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

#[cfg(feature = "highlight")]
impl BuiltIn {
    /// Clone nodes and highlight them to LSP accept SemanticTokens.
    pub fn to_semantic(&self) -> Vec<SemanticToken> {
        let nodes = self.nodes.clone();
        let mut tokens = vec![];
        for node in nodes {
            color_nodes(node, &mut tokens);
        }

        let mut tokens: Vec<_> = tokens
            .into_iter()
            .map(|abs_t| {
                (
                    abs_t.line,
                    abs_t.start,
                    abs_t.len,
                    abs_t.ty.to_index(),
                    abs_t.mods,
                )
            })
            .collect();

        tokens.sort_unstable_by_key(|&(line, start, ..)| (line, start));

        let (mut prev_line, mut prev_start) = (0, 0);

        tokens
            .into_iter()
            .map(|(line, start, length, token_type, mods)| {
                let delta_line = line - prev_line;
                // On the same line the start is relative to the previous token.
                let delta_start = if delta_line == 0 {
                    start - prev_start
                } else {
                    start
                };
                (prev_line, prev_start) = (line, start);
                SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset: mods,
                }
            })
            .collect()
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
            #[cfg(feature = "highlight")]
            let len_before = nodes.len();

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

            #[cfg(feature = "highlight")]
            // NOTE: `Block` and `InlineMark` don't use `nth_len` for skipping tokens, so all tokens
            // around them will be pushed into `BuiltIn.nodes`. This highlight feature is NOT
            // enabled by default.
            if nodes.len() == len_before {
                nodes.push(Self::Token(cursor.token.clone()));
            }

            if cursor.next().is_none() {
                break;
            }
        }

        nodes
    }

    #[cfg(feature = "debug")]
    #[warn(clippy::arbitrary_source_item_ordering)]
    /// An info API for test harnnes (Debug only).
    pub fn info(&self) -> (&'static str, Span) {
        match self {
            Node::Block(b) => ("Block", b.span),
            Node::Comment(c) => ("Comment", c.span),
            #[cfg(feature = "highlight")]
            Node::InlineMark(m) => ("InlineMark", m.span),
            Node::Token(t) => ("Token", t.span),
        }
    }
}
