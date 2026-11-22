//! # Mapping
//!
//! Build HashMap for parsing Blocks.
use std::collections::HashMap;

use crate::{Token, TokenKind};

pub type Map = HashMap<usize, usize>;

pub fn build_map(tokens: &[Token]) -> Map {
    let mut stack = vec![];
    let mut map = Map::new();
    for (i, t) in tokens.iter().enumerate() {
        match t.ty {
            TokenKind::LeftBlock => stack.push(i),
            TokenKind::LeftBlockEnd => {
                // always close on like this, 2 is an const
                // `[[/name]]`
                //  ^^^ block_end
                //     ^^^^ 1
                //         ^^ 2
                // index of `]]` is `block_end + 2`
                if let Some(open) = stack.pop() {
                    map.insert(open, i + 2); // [[ -> ]]
                }
            }
            _ => {}
        }
    }

    map
}
