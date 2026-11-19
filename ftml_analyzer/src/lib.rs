#![warn(clippy::all)]
#![warn(unused_lifetimes)]
pub use ftml::parsing::Token as TokenKind;

pub mod cursor;
pub mod ftml_ast;
pub mod nodes;

pub use cursor::Cursor;
pub use ftml_ast::{Span, Token};
