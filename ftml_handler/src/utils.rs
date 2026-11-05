use ftml::parsing::{ParseErrorKind, Token as TokenTy};

use crate::ftml_ast::{Diagnostic, Token};

/// Check if token span has diagnostic.
pub(crate) fn has_diagnostic(token: &Token, diagnostic: &Diagnostic) -> bool {
    let token_span = &token.span;
    let diagnostic_span = &diagnostic.span;
    token_span.start == diagnostic_span.start && token_span.end == diagnostic_span.end
}

pub(crate) fn check_diagnostic_without_include(token: &Token, diagnostic: &Diagnostic) -> bool {
    let token_span = &token.span;
    let diagnostic_span = &diagnostic.span;
    token_span.start == diagnostic_span.start
        && token_span.end == diagnostic_span.end
        && matches!(diagnostic.ty, ParseErrorKind::InvalidInclude)
}

/// Check the token type for inlinemark.
pub(crate) fn is_inline_mark_token(ty: TokenTy) -> bool {
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

/// Check the token type for inlineblock.
pub(crate) fn is_inline_block_token(ty: TokenTy) -> bool {
    matches!(ty, TokenTy::LeftBlock | TokenTy::LeftBlockStar)
}
