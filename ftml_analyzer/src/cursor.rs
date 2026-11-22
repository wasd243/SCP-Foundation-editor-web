mod update;

use crate::{Token, TokenKind};

#[derive(Debug)]
/// # Cursor
///
/// For parsing span to BuiltIn AST nodes.
/// The cursor will read information before/after token to provide context.
///
/// All context in `Cursor` won't move value and will remains read-only borrows.
pub struct Cursor<'c> {
    pub index: usize,
    /// Token before cursor pointing.
    pub before: Option<&'c Token>,
    /// Token cursor is pointing.
    pub token: &'c Token,
    /// Token after cursor pointing.
    pub after: Option<&'c Token>,
    /// All the tokens after cursor pointing one.
    pub rest: Option<&'c [Token]>,
    /// Check that is cursor in `[[ ]]` context.
    block_depth: usize,
    /// This field only use for checking in block attributes equals.
    /// Always false while initializing cursor.
    /// Private because it remains `true` even if the cursor is NOT in block.
    after_equals: bool,
    /// This field only use for checking in block attributes "double quotes".
    /// Always false while initializing cursor.
    is_string: bool,
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

        // Automatically check cursor status.
        let mut block_depth = 0;
        add_block_depth(token.ty, &mut block_depth);

        Some(Self {
            index: 0,
            before,
            token,
            after,
            rest,
            block_depth,
            after_equals: false,
            is_string: false,
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

    /// A manually nth implementation for builtin nodes.
    pub fn nth_len(&mut self, len: usize) {
        for _ in 1..len {
            self.next();
        }
    }

    /// Check cursor is in block.
    pub fn is_in_block(&self) -> bool {
        self.block_depth > 0
    }

    /// Check that the cursor is after `=` context.
    pub fn is_value(&self) -> bool {
        self.is_in_block() && self.after_equals
    }

    /// Check that the cursor is in `"string"`.
    pub fn is_string(&self) -> bool {
        self.is_string || (self.token.ty == TokenKind::DoubleQuote && self.is_in_block())
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
            self.index += 1;
            self.update();
            return Some(self.token);
        }

        self.before = Some(self.token);
        self.token = self.after?;

        let (after, rest) = self.rest?.split_first()?;
        self.after = Some(after);
        self.rest = if rest.is_empty() { None } else { Some(rest) };

        self.index += 1;
        self.update();

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

/// +1 when current is `[[`.
fn add_block_depth(ty: TokenKind, depth: &mut usize) {
    if ty == TokenKind::LeftBlock {
        *depth += 1;
    }
}

/// -1 when before is `]]`.
fn minus_block_depth(ty: TokenKind, depth: &mut usize) {
    if ty == TokenKind::RightBlock {
        *depth = depth.saturating_sub(1);
    }
}
