use regex::Regex;
use serde_json::Value;

use crate::ftml_ast::{ParseToken, ValueIterDirection};

use super::{BLOCK_CLOSE, BLOCK_END, IDENTIFIER};

fn slice_of(t: &Value) -> &str {
    t["slice"].as_str().unwrap_or("")
}

/// Check whether the token at `i` starts a `[[/name]]` sequence,
/// where `name` matches `identifier_re`.
pub(super) fn check_end(p: &ParseToken, i: usize, identifier_slice_re: &Regex) -> bool {
    // token at `i` itself must be the end-block opener
    let is_opener = p
        .tokens
        .as_array()
        .and_then(|a| a.get(i))
        .is_some_and(|t| t["token"] == BLOCK_CLOSE);
    if !is_opener {
        return false;
    }

    let mut iter = p.value_iter(i, ValueIterDirection::Forward);

    // identifier matching the regex
    let Some(t) = iter.next() else { return false };
    if t["token"] != IDENTIFIER || !identifier_slice_re.is_match(slice_of(t)) {
        return false;
    }

    // closing bracket
    let Some(t) = iter.next() else { return false };
    t["token"] == BLOCK_END
}
