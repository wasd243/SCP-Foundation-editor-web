#![allow(unused)]
mod types;

use serde::Serialize;

use crate::{BuiltIn, nodes::Node};

pub use types::{MODIFIERS, TYPES};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct AbsoluteToken {
    pub line: u32,
    pub start: u32,
    pub len: u32,
    pub ty: HighlightKind,
    pub mods: u32,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[repr(u32)]
pub enum HighlightKind {
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
    /// ^^
    Superscript = 15,
    /// ,,
    Subscript = 16,
    /// [[ ]] [[/ [[*
    Operator = 17,
    /// https://example.com
    Link = 18,
    /// +++ head
    Heading = 19,
}
