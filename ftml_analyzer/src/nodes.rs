mod comment;

use comment::Comment;

use serde::Serialize;

use crate::{
    Cursor, Token,
    ftml_ast::{Diagnostic, ParseToken},
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
/// # Node
///
/// This enum represents individual nodes in the LSP builtin AST.
/// Each node has a struct with the same name.
/// This enum is constructed in `BuiltIn`.
pub enum Node {
    Comment(Comment),
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
        let mut nodes: Vec<Self> = vec![];

        loop {
            if let Some(comment) = Comment::parse(&cursor) {
                cursor.nth(comment.len());
                nodes.push(Self::Comment(comment));
            }

            if cursor.after.is_none() {
                break;
            }
        }

        nodes
    }
}
