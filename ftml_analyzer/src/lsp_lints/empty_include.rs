use ftml::parsing::Token as TokenTy;
use regex::Regex;

use std::sync::LazyLock;

use crate::{Span, ftml_ast::Token};

use super::{LintKind, LspLint};

static INCLUDE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?i)include$").unwrap());
const NAME: &str = "empty_include";
const MSG: &str = "Consider remove this include block or wrap with @@ raw marks";

/// Capture empty `[[include]]`
///
/// NOTE: FTML's `no_such_include` warnings are NOT reliable for
/// includes, so implement this manualy.
pub(super) fn check(tokens: &[Token]) -> Vec<LspLint> {
    let significant: Vec<&Token> = tokens
        .iter()
        .filter(|t| !matches!(t.ty, TokenTy::Whitespace))
        .collect();

    significant
        .windows(3)
        .filter_map(|w| match w {
            [open, name, close]
                if matches!(open.ty, TokenTy::LeftBlock)
                    && matches!(name.ty, TokenTy::Identifier)
                    && INCLUDE.is_match(&name.slice)
                    && matches!(close.ty, TokenTy::RightBlock) =>
            {
                let span = Span {
                    start: open.span.start,
                    end: close.span.end,
                };
                Some(LspLint::new(
                    NAME,
                    span,
                    MSG.to_owned(),
                    LintKind::Correctness,
                ))
            }
            _ => None,
        })
        .collect()
}
