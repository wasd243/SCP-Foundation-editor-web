use crate::{Token, TokenKind};

#[derive(Debug)]
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
    pub rest: Option<&'c [Token]>,
}

impl<'c> Cursor<'c> {
    // NOTE: For constants in `Cursor`:
    // These constants are used for calculating span/index/etc.
    // `Cursor` is used for pointing them, so these constants are written as implementations
    // of `Cursor`.

    ///The token count of a mark with no content (start and end marks only).
    pub const START_AND_END_MARK: usize = 2;

    /// Build a new cursor from tokens.
    pub fn new(tokens: &'c [Token]) -> Option<Self> {
        let before: Option<&Token> = None;

        let (token, rest) = tokens.split_first()?;

        let (after, rest) = match rest.split_first() {
            Some((after, rest)) => (Some(after), Some(rest)),
            None => (None, None),
        };

        Some(Self {
            before,
            token,
            after,
            rest,
        })
    }

    /// Search all the tokens from `before` to `rest`.
    pub fn contains_token(&self, token_kind: TokenKind) -> bool {
        self.before.is_some_and(|t| t.ty == token_kind)
            || self.token.ty == token_kind
            || self.after.is_some_and(|t| t.ty == token_kind)
            || self
                .rest
                .is_some_and(|ts| ts.iter().any(|t| t.ty == token_kind))
    }
}

impl<'c> Iterator for Cursor<'c> {
    type Item = &'c Token;

    /// Push the whole Cursor to next token.
    fn next(&mut self) -> Option<Self::Item> {
        // make sure that we have another token to push
        self.after?;

        // return last token if we have no more
        if self.rest.is_none() {
            self.before = Some(self.token); // also update for other fields
            self.token = self.after?;
            self.after = None;
            return Some(self.token);
        }

        self.before = Some(self.token);
        self.token = self.after?;

        let (after, rest) = self.rest?.split_first()?;
        self.after = Some(after);
        self.rest = if rest.is_empty() { None } else { Some(rest) };

        Some(self.token)
    }

    /// Get last item.
    fn last(self) -> Option<Self::Item> {
        self.rest?.last()
    }
}
