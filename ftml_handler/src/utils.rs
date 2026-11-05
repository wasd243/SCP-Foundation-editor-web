use ftml::parsing::Token as TokenTy;

use crate::ftml_ast::{Diagnostic, Token};

/// Check if token span has diagnostic.
pub fn has_diagnostic(token: &Token, diagnostic: &Diagnostic) -> bool {
    let token_span = &token.span;
    let diagnostic_span = &diagnostic.span;
    token_span.start == diagnostic_span.start && token_span.end == diagnostic_span.end
}

/// Check the token type for inlinemark.
pub fn is_inline_mark_token(ty: TokenTy) -> bool {
    matches!(
        ty,
        TokenTy::Bold
            | TokenTy::Italics
            | TokenTy::Underline
            | TokenTy::Superscript
            | TokenTy::Subscript
            | TokenTy::DoubleDash
            | TokenTy::DoubleTilde
            | TokenTy::Raw
            | TokenTy::LeftMonospace
    )
}
