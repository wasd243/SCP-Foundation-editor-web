use crate::ftml_ast::Span;
use ftml::parsing::ExtractedToken;
use serde::{Deserialize, Serialize};

/// A lsp version of extracted token from ftml.
#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    /// The lsp mark for `include` blocks.
    #[serde(rename = "include-fold-mark", skip_serializing_if = "Option::is_none")]
    pub include_fold_mark: Option<IncludeFoldMark>,
    /// The lsp mark for sub languages highlights.
    #[serde(rename = "sub-lang-mark", skip_serializing_if = "Option::is_none")]
    pub sub_lang_mark: Option<SubLangMark>,
    /// The slice of the original text.
    pub slice: String,
    /// The span of the original text.
    pub span: Span,
    /// The token type.
    pub ty: ftml::parsing::Token,
    /// `true` if combined tokens into one token slice, `false` as default.
    #[serde(skip)]
    pub from_combination: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum IncludeFoldMark {
    IncludeStart,
    IncludeEnd,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SubLangMark {
    Html,
    Css,
}

impl Token {
    pub fn from_extracted_tokens(extracted_token: ExtractedToken) -> Self {
        Self {
            // These will be filled in ftml block identifier
            include_fold_mark: None,
            sub_lang_mark: None,

            slice: extracted_token.slice.to_string(),
            span: Span {
                start: extracted_token.span.start,
                end: extracted_token.span.end,
            },
            ty: extracted_token.token,
            from_combination: false,
        }
    }
}
