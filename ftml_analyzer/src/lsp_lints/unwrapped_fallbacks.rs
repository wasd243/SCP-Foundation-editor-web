use ftml::parsing::ParseErrorKind;

use crate::ftml_ast::Diagnostic;

use super::{LintKind, LspLint};

const NAME: &str = "unwrapped_fallback";
const MSG: &str = "Consider adding `@@` `@@` raw wrappings";

/// `NoRulesMatch` mostly start at position like:
///
/// ```ftml,ignore
/// [[no such rules]]
/// ^^             ^^
/// ```
///
/// So we could calculate span by adding `]]` span to `[[` span.
/// And then linting the span, convert it to LSP diagnostics.
///
/// # NOTE:
/// `ftml`'s diagnostics are **NOT** reliable. Sometime it'll generate unmatched `ParseError` in
/// several edge cases, so we highlight the whole line rather than on the `[[` `]]` span.
pub(super) fn check(diagnostics: &[Diagnostic]) -> Vec<LspLint> {
    diagnostics
        .iter()
        .filter(|d| d.ty == ParseErrorKind::NoRulesMatch)
        .map(|d| LspLint::new(NAME, d.span, MSG.to_owned(), LintKind::Hint))
        .collect()
}
