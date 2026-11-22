#![warn(clippy::all)]
#![warn(unused_lifetimes)]
pub use ftml::parsing::Token as TokenKind;

pub mod cursor;
pub mod ftml_ast;
pub mod mapping;
pub mod nodes;

pub use cursor::Cursor;
pub use ftml_ast::{Span, Token};

use crate::{ftml_ast::ParseToken, nodes::BuiltIn};

/// Get the BuiltIn AST.
pub fn get_ast(src: &str) -> BuiltIn {
    let parse_token = ParseToken::from(src);
    BuiltIn::from(parse_token)
}
