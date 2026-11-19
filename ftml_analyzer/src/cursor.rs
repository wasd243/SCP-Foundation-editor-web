use serde::Serialize;

use crate::Token;

#[derive(Debug, Serialize)]
/// # Cursor
///
/// For parsing span to BuiltIn AST nodes.
/// The cursor will read information before/after token to provide context.
///
/// All context in `Cursor` won't move value and will remains read-only borrows.
pub struct Cursor<'c> {
    /// The token before cursor pointing one.
    pub before: Option<&'c Token>,
    /// The token cursor is pointing.
    pub token: &'c Token,
    /// The token after cursor pointing one.
    pub after: Option<&'c Token>,
    /// All the tokens after cursor pointing one.
    pub rest: Option<Vec<&'c Token>>,
}

impl<'c> Cursor<'c> {
    pub fn new(tokens: &'c [Token]) -> Option<Self> {
        if tokens.is_empty() {
            return None;
        }

        let before: Option<&Token> = None;
        let token = &tokens[0];

        let mut iter = tokens.iter();

        let after = iter.next();

        if after.is_some() {
            let rest: Vec<&Token> = iter.collect();
            Some(Self {
                before,
                token,
                after,
                rest: Some(rest),
            })
        } else {
            Some(Self {
                before,
                token,
                after,
                rest: None,
            })
        }
    }
}
