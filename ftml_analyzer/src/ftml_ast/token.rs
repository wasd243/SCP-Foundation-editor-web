use crate::ftml_ast::Span;
use ftml::parsing::{ExtractedToken, Token as TokenTy};
use serde::{Deserialize, Serialize};

/// A lsp version of extracted token from ftml.
#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    /// The slice of the original text.
    pub slice: String,
    /// The span of the original text.
    pub span: Span,
    /// The token type.
    pub ty: TokenTy,
}

impl Token {
    pub fn from_extracted_tokens_with_span(extracted_token: ExtractedToken, span: Span) -> Self {
        Self {
            slice: extracted_token.slice.to_string(),
            span,
            ty: extracted_token.token,
        }
    }
}
