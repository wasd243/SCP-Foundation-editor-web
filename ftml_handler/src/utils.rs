use crate::ftml_ast::{Diagnostic, Token};

/// check if token span has diagnostic
pub fn has_diagnostic(token: &Token, diagnostic: &Diagnostic) -> bool {
    let token_span = &token.span;
    let diagnostic_span = &diagnostic.span;
    if token_span.start == diagnostic_span.start && token_span.end == diagnostic_span.end {
        return true;
    }
    false
}
