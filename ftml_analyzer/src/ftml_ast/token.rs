use crate::{
    ftml_ast::Span,
    lsp_highlight::{AbsoluteSpan, HighlightTy, ToAbsoluteSpan},
};
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

impl ToAbsoluteSpan for Token {
    fn to_abs_span(&self) -> Option<AbsoluteSpan> {
        Some(AbsoluteSpan {
            line: self.span.start.0,
            start: self.span.start.1,
            len: self.span.end.1 - self.span.start.1,
            ty: match_token_ty(&self.ty)?,
            // TODO: mods are temporarily a placeholder
            mods: 0,
        })
    }
}

fn match_token_ty(ty: &TokenTy) -> Option<HighlightTy> {
    match ty {
        TokenTy::Quote => Some(HighlightTy::Quote),
        TokenTy::Identifier => Some(HighlightTy::KeyWord),
        _ => None,
    }
}
