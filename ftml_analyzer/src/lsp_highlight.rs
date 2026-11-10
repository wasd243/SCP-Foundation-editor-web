use serde::Serialize;

use crate::lsp_builtin::BuiltIn;

/// For converting span -> AbsoluteSpan.
pub trait ToAbsoluteSpan {
    fn to_abs_span(&self) -> Option<AbsoluteSpan>;
}

/// For converting mutiple spans -> Vec<AbsoluteSpan>.
pub trait ToAbsoluteSpans {
    fn to_abs_spans(&self) -> Vec<AbsoluteSpan>;
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
    /// div, collapsible, etc.
    KeyWord = 0,
    /// =
    Equals = 1,
    /// >
    Quote = 2,
    /// **
    Bold = 3,
    /// //
    Italics = 4,
    /// __
    Underline = 5,
    /// --
    StrikeThrough = 6,
    /// {{}}
    Monospace = 7,
    /// @@
    Raw = 8,
    /// [!----]
    Comment = 9,
    /// div class, collapsible show,hide, etc.
    Attr = 10,
    /// |
    Pipe = 11,
    /// ""
    String = 12,
    /// |attr=Value
    Value = 13,
    /// include :component:image-block
    PathBuffer = 14,
    // TODO: add more as needed, keep in sync with the legend
}

impl AbsoluteSpanTree {
    /// Build a `AbsoluteSpanTree` from `LspBuiltIn` tree.
    pub fn from_builtin_tree(tree: Vec<BuiltIn>) -> Self {
        let mut spans = Vec::new();

        for node in tree {
            match node {
                BuiltIn::Token(t) => spans.extend(t.to_abs_span()),
                BuiltIn::InLineMarks(m) => spans.extend(m.to_abs_spans()),
                BuiltIn::InLineBlocks(b) => spans.extend(b.to_abs_spans()),
                BuiltIn::Blocks(b) => spans.extend(b.to_abs_spans()),
            }
        }

        Self { tree: spans }
    }
}
