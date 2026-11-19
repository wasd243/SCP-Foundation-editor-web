#![warn(clippy::all)]
#![warn(unused_lifetimes)]

pub mod cursor;
pub mod ftml_ast;

pub use crate::ftml_ast::{Span, Token};
