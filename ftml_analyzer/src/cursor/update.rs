use crate::TokenKind;

use super::{Cursor, add_block_depth, minus_block_depth};

impl<'c> Cursor<'c> {
    pub(super) fn update(&mut self) {
        self.update_string();
        // do not update `=` or `depth` in string
        if !self.is_string() {
            self.update_eq();
            self.update_depth();
        }
    }

    /// Update block depth.
    fn update_depth(&mut self) {
        add_block_depth(self.token.ty, &mut self.block_depth);
        if let Some(before) = self.before {
            minus_block_depth(before.ty, &mut self.block_depth);
        }
    }

    /// Update self eq field.
    fn update_eq(&mut self) {
        if let Some(before) = self.before {
            match before.ty {
                TokenKind::Equals => self.after_equals = true,
                TokenKind::RightBlock | TokenKind::Whitespace | TokenKind::Pipe => {
                    self.after_equals = false
                }
                _ => {}
            }
        }
    }

    /// Update self `is_string` field.
    fn update_string(&mut self) {
        if let Some(before) = self.before
            && before.ty == TokenKind::DoubleQuote
            && self.is_in_block()
        {
            self.is_string = !self.is_string;
        }
    }
}
