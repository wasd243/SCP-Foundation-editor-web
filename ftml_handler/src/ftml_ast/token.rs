use crate::ftml_ast::Span;
use ftml::parsing::ExtractedToken;
use serde::{Deserialize, Serialize};

/// A lsp version of extracted token from ftml.
#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    /// The slice of the original text.
    pub slice: String,
    /// The span of the original text.
    pub span: Span,
    /// The token type.
    pub ty: ftml::parsing::Token,
}

impl Token {
    pub fn from_extracted_tokens(extracted_token: ExtractedToken) -> Self {
        Self {
            slice: extracted_token.slice.to_string(),
            span: Span {
                start: extracted_token.span.start,
                end: extracted_token.span.end,
            },
            ty: extracted_token.token,
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        let slice = String::new();
        let span = Span::default();
        let ty = ftml::parsing::Token::Whitespace;

        Self { slice, span, ty }
    }
}
