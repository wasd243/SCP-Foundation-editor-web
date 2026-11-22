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
    /// Check that is cursor in `[[ ]]` context.
    block_depth: usize,
}

impl<'c> Cursor<'c> {
    // NOTE: For constants in `Cursor`:
    // These constants are used for calculating span/index/etc.
    // `Cursor` is used for pointing them, so these constants are written as implementations
    // of `Cursor`.

    ///The token count of a mark with no content (start and end marks only).
    pub const START_AND_END_MARK: usize = 2;

    /// The token count of a mark `]]` because the cursor won't count it into `block_depth`.
    pub const BLOCK_END_MARK: usize = 1;

    /// Build a new cursor from tokens.
    pub fn new(tokens: &'c [Token]) -> Option<Self> {
        let before: Option<&Token> = None;

        let (token, rest) = tokens.split_first()?;

        let (after, rest) = match rest.split_first() {
            Some((after, rest)) => (Some(after), Some(rest)),
            None => (None, None),
        };

        // Automatically check cursor status.
        let mut block_depth = 0;
        match_block_tokens(token.ty, &mut block_depth);

        Some(Self {
            before,
            token,
            after,
            rest,
            block_depth,
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

    /// Check cursor is in block.
    pub fn is_in_block(&self) -> bool {
        self.block_depth > 0
    }

    /// A manually nth implementation for builtin nodes.
    pub fn nth_len(&mut self, len: usize) {
        for _ in 1..len {
            self.next();
        }
    }

    /// Update block depth.
    fn update_depth(&mut self) {
        match_block_tokens(self.token.ty, &mut self.block_depth);
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
            self.update_depth();
            return Some(self.token);
        }

        self.before = Some(self.token);
        self.token = self.after?;

        let (after, rest) = self.rest?.split_first()?;
        self.after = Some(after);
        self.rest = if rest.is_empty() { None } else { Some(rest) };

        self.update_depth();

        Some(self.token)
    }

    /// Get last item.
    fn last(self) -> Option<Self::Item> {
        self.rest
            .and_then(|r| r.last())
            .or(self.after)
            .or(Some(self.token))
    }
}

/// Match and update block depth.
///
/// NOTE: `is_in_block()` only contains `[[` not `]]`,
/// so the status is including `[[` but excluding `]]`.
fn match_block_tokens(ty: TokenKind, depth: &mut usize) {
    match ty {
        TokenKind::LeftBlock => *depth += 1,
        TokenKind::RightBlock => *depth = depth.saturating_sub(1),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::ftml_ast::ParseToken;

    use super::*;

    const FTML: &str = "[[footnote]]something[[/footnote]]";

    #[test]
    fn cursor_works() {
        let tokens = ParseToken::from(FTML);
        let mut cursor = Cursor::new(&tokens.tokens).unwrap();
        assert!(cursor.is_in_block());
        cursor.next();
        assert!(cursor.is_in_block());
        cursor.next();
        assert!(!cursor.is_in_block());
    }
}
