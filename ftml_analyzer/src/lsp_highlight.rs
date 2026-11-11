pub mod types;

use lsp_types::{SemanticToken, SemanticTokenType};
use serde::Serialize;

use crate::{lsp_builtin::BuiltIn, lsp_highlight::types::TYPES, utils::highlight::color_nodes};

/// For converting span -> AbsoluteSpan.
pub trait ToAbsoluteSpan {
    fn to_abs_span(&self) -> Option<AbsoluteSpan>;
}

/// For converting multiple spans -> Vec<AbsoluteSpan>.
pub trait ToAbsoluteSpans {
    fn to_abs_spans(&self) -> Vec<AbsoluteSpan>;
}

#[derive(Debug, Serialize, Clone)]
pub struct AbsoluteSpanTree {
    tree: Vec<AbsoluteSpan>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AbsoluteSpan {
    pub line: u32,
    pub start: u32,
    pub len: u32,
    pub ty: HighlightTy,
    pub mods: u32,
}

#[derive(Debug, Serialize, Clone)]
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
    /// ^^
    Superscript = 15,
    /// ,,
    Subscript = 16,
    /// [[ ]] [[/ [[*
    Operator = 17,
    // TODO: add more as needed, keep in sync with the legend
}

impl HighlightTy {
    fn to_index(&self) -> u32 {
        let ty = match self {
            Self::KeyWord => SemanticTokenType::KEYWORD,
            Self::Equals | Self::Pipe | Self::Quote | Self::Operator => SemanticTokenType::OPERATOR,
            Self::Comment => SemanticTokenType::COMMENT,
            Self::Attr => SemanticTokenType::PROPERTY,
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
        };

        //  NOTE: The slice and `ty` come from the same list, so this cannot fail.
        TYPES.iter().position(|t| *t == ty).unwrap() as u32
    }
}

impl AbsoluteSpanTree {
    /// Build a `AbsoluteSpanTree` from `LspBuiltIn` tree.
    pub fn from_builtin_tree(tree: Vec<BuiltIn>) -> Self {
        let mut spans = Vec::new();

        for node in tree {
            color_nodes(&node, &mut spans);
        }

        Self { tree: spans }
    }

    /// Convert `AbsoluteSpanTree` IR into LSP acceptable type.
    pub fn into_semantic_tokens(self) -> Vec<SemanticToken> {
        let mut spans: Vec<_> = self
            .tree
            .into_iter()
            .filter_map(|s| Some((s.line, s.start, s.len, s.ty.to_index(), s.mods)))
            .collect();
        spans.sort_unstable_by_key(|&(line, start, ..)| (line, start));

        let (mut prev_line, mut prev_start) = (0, 0);
        spans
            .into_iter()
            .map(|(line, start, length, token_type, mods)| {
                let delta_line = line - prev_line;
                // On the same line the start is relative to the previous token.
                let delta_start = if delta_line == 0 {
                    start - prev_start
                } else {
                    start
                };
                (prev_line, prev_start) = (line, start);
                SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset: mods,
                }
            })
            .collect()
    }
}
