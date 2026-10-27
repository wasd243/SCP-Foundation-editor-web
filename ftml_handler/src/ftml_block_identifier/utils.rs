use regex::Regex;

use crate::ftml_ast::ParseToken;

use super::{IDENTIFIER, LEFT_BLOCK, LEFT_BLOCK_END, RIGHT_BLOCK};

/// Check whether the token at `i` starts a `[[/name]]` sequence,
/// where `name` matches `identifier_re`.
pub(super) fn check_end(
    parse_token: &ParseToken,
    index: usize,
    identifier_slice_re: &Regex,
) -> bool {
    let is_opener = parse_token
        .tokens
        .iter()
        .any(|token| matches!(token.ty, LEFT_BLOCK_END));

    if !is_opener {
        return false;
    }

    let mut iter = parse_token.value_iter(index);

    // identifier matching the regex
    let Some((_, token)) = iter.next() else {
        return false;
    };
    if token.ty != IDENTIFIER || !identifier_slice_re.is_match(token.slice.as_str()) {
        return false;
    }

    // closing bracket
    let Some((_, token)) = iter.next() else {
        return false;
    };
    token.ty == RIGHT_BLOCK
}

pub(super) fn is_open(parse_token: &ParseToken, index: usize) -> bool {
    if let Some(token) = parse_token.tokens.get(index)
        && token.ty == LEFT_BLOCK
    {
        return true;
    }
    false
}
