mod types;

use lsp_types::SemanticTokenType;
use serde::Serialize;

pub use types::{MODIFIERS, TYPES};

use crate::nodes::Node;

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

impl HighlightKind {
    pub(crate) fn to_index(self) -> u32 {
        let ty = match self {
            Self::KeyWord => SemanticTokenType::KEYWORD,
            Self::Equals | Self::Pipe | Self::Quote | Self::Operator => SemanticTokenType::OPERATOR,
            Self::Comment => SemanticTokenType::COMMENT,
            Self::Attr | Self::Heading => SemanticTokenType::PROPERTY,
            Self::String => SemanticTokenType::STRING,
            Self::Value => SemanticTokenType::ENUM_MEMBER,
            Self::PathBuffer => SemanticTokenType::NAMESPACE,
            Self::Raw => SemanticTokenType::REGEXP,
            Self::Bold => SemanticTokenType::new("ftmlBold"),
            Self::Italics => SemanticTokenType::new("ftmlItalic"),
            Self::Underline => SemanticTokenType::new("ftmlUnderline"),
            Self::StrikeThrough => SemanticTokenType::new("ftmlStrike"),
            Self::Monospace => SemanticTokenType::new("ftmlMonospace"),
            Self::Superscript => SemanticTokenType::new("ftmlSuperscript"),
            Self::Subscript => SemanticTokenType::new("ftmlSubscript"),
            Self::Link => SemanticTokenType::new("ftmlLinkUrl"),
        };

        // NOTE: The slice and `ty` come from the same list, so this cannot fail.
        TYPES.iter().position(|t| *t == ty).unwrap() as u32
    }
}

/// Color each node by their implementations and push back.
pub(crate) fn color_nodes(node: Node, abs_tokens: &mut Vec<AbsoluteToken>) {
    match node {
        Node::Block(b) => abs_tokens.extend(b.to_abs_tokens()),
        Node::Comment(c) => abs_tokens.extend(c.to_abs_tokens()),
        Node::InlineMark(im) => abs_tokens.push(im.to_abs_span()),
        Node::Token(t) => abs_tokens.extend(t.to_abs_token()),
    }
}
