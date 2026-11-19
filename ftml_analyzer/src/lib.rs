#![warn(clippy::all)]
#![warn(unused_lifetimes)]

pub mod cursor;
pub mod ftml_ast;

pub use cursor::Cursor;
pub use ftml_ast::{Span, Token};
