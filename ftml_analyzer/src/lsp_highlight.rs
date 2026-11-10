use serde::Serialize;

use crate::lsp_builtin::BuiltIn;

pub trait ToAbsoluteSpan {
    fn to_abs_span(&self) -> Option<AbsoluteSpan>;
}

#[derive(Debug, Serialize)]
pub struct AbsoluteSpanTree {
    tree: Vec<AbsoluteSpan>,
}

#[derive(Debug, Serialize)]
pub struct AbsoluteSpan {
    pub line: u32,
    pub start: u32,
    pub len: u32,
    pub ty: HighlightTy,
    pub mods: u32,
}

#[derive(Debug, Serialize)]
#[repr(u32)]
pub enum HighlightTy {
    KeyWord = 0,
    Equals = 1,
    Quote = 2,
    // TODO: add more as needed, keep in sync with the legend
}

impl AbsoluteSpanTree {
    /// Build a `AbsoluteSpanTree` from `LspBuiltIn` tree.
    pub fn from_builtin_tree(tree: Vec<BuiltIn>) -> Self {
        let mut spans = Vec::new();

        for node in tree {
            match node {
                BuiltIn::Token(t) => spans.extend(t.to_abs_span()),
                _ => {} // TODO: implement for other types.
            }
        }

        Self { tree: spans }
    }
}
