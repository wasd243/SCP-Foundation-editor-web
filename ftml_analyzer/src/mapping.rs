//! # Mapping
//!
//! Build HashMap for parsing Blocks.
use std::collections::HashMap;

use crate::{Token, TokenKind};

pub type Map = HashMap<usize, usize>;

pub fn build_map(tokens: &[Token]) -> Map {
    let mut stack: Vec<(usize, &str)> = vec![];
    let mut map: Map = Map::new();
    for (i, t) in tokens.iter().enumerate() {
        match t.ty {
            TokenKind::LeftBlock => {
                // Name is the token right after `[[`.
                if let Some(name) = tokens.get(i + 1) {
                    stack.push((i, &name.slice));
                }
            }
            TokenKind::LeftBlockEnd => {
                // always close on like this, 2 is an const
                // `[[/name]]`
                //  ^^^ block_end
                //     ^^^^ 1
                //         ^^ 2
                // index of `]]` is `block_end + 2`
                let Some(name) = tokens.get(i + 1) else {
                    continue;
                };
                let close_name: &str = &name.slice;
                // Find the nearest open tag with the same name, not just the stack top.
                if let Some(pos) = stack
                    .iter()
                    .rposition(|(_, n)| n.eq_ignore_ascii_case(close_name))
                {
                    map.insert(stack[pos].0, i + 2); // [[ -> ]]
                    // Drop the match and every unclosed tag above it.
                    stack.truncate(pos);
                }
            }
            _ => {}
        }
    }

    map
}
