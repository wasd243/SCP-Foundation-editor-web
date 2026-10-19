mod regex;
mod utils;

use serde_json::Value;

use crate::ftml_ast::{ParseToken, ValueIterDirection};
use crate::ftml_block_identifier::regex::*;
use crate::ftml_block_identifier::utils::check_end;
use crate::value_iter_next;

pub(crate) const IDENTIFIER: &str = "identifier";
pub(crate) const BLOCK_START: &str = "left-block";
pub(crate) const BLOCK_END: &str = "right-block";
pub(crate) const BLOCK_CLOSE: &str = "left-block-end";

type Marks = Vec<(usize, &'static str)>;

impl ParseToken {
    pub fn mark_sublanguage_blocks(&mut self) {
        let len = self.tokens.as_array().map_or(0, Vec::len);
        let mut marks: Marks = Vec::new();

        // Phase 1: read-only, collect marks
        for i in 0..len {
            module_css(self, i, &mut marks);
            html(self, i, &mut marks);
        }

        // Phase 2: write back
        if let Some(arr) = self.tokens.as_array_mut() {
            for (i, role) in marks {
                arr[i]["lsp-mark"] = Value::from(role);
            }
        }
    }
}

/// mark CSS modules like below in token stream:
/// ```ignore
/// [[module css]]
/// /* CSS */
/// [[/module]]
/// ```
fn module_css(p: &ParseToken, i: usize, marks: &mut Marks) {
    let Some(t) = p.tokens.as_array().and_then(|tokens| tokens.get(i)) else {
        return;
    };
    if t["token"] != BLOCK_START {
        return;
    }

    let mut value_iter = p.value_iter_indexed(i, ValueIterDirection::Forward);

    value_iter_next!(t, value_iter);
    let slice = t["slice"].as_str().unwrap_or("");
    if t["token"] != IDENTIFIER || !MODULE.is_match(slice) {
        return;
    }

    value_iter_next!(t, value_iter);
    let slice = t["slice"].as_str().unwrap_or("");
    if t["token"] != IDENTIFIER || !CSS.is_match(slice) {
        return;
    }

    value_iter_next!(t, value_iter);
    if t["token"] != BLOCK_END {
        return;
    }

    for (idx, _) in value_iter {
        if check_end(p, idx, &MODULE) {
            break;
        }
        marks.push((idx, "css"));
    }
}

const MAX_HTML_BLOCK_ALIAS_TOKEN: u8 = 100;
/// Identify html blocks in token streams.
fn html(p: &ParseToken, i: usize, marks: &mut Marks) {
    let Some(t) = p.tokens.as_array().and_then(|tokens| tokens.get(i)) else {
        return;
    };
    if t["token"] != BLOCK_START {
        return;
    }

    let mut value_iter = p.value_iter_indexed(i, ValueIterDirection::Forward);

    value_iter_next!(t, value_iter);
    let slice = t["slice"].as_str().unwrap_or("");
    if t["token"] != IDENTIFIER || !HTML.is_match(slice) {
        return;
    }

    value_iter_next!(t, value_iter);
    if t["token"] != BLOCK_END {
        for _ in 0..MAX_HTML_BLOCK_ALIAS_TOKEN {
            value_iter_next!(t, value_iter);

            if t["token"] == BLOCK_END {
                break;
            }
        }
    }

    for (idx, _) in value_iter {
        if check_end(p, idx, &HTML) {
            break;
        }
        marks.push((idx, "html"));
    }
}
