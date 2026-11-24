use crate::{
    ftml_ast::Span,
    highlight::{AbsoluteToken, HighlightKind},
};
use ftml::parsing::{ExtractedToken, Token as TokenTy};
use serde::{Deserialize, Serialize};

/// A lsp version of extracted token from ftml.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

    /// Whether the range from `self` to `other` crosses lines.
    pub fn is_cross_line_with(&self, other: &Self) -> bool {
        self.span.start.0 != other.span.end.0
    }

    #[cfg(feature = "highlight")]
    /// Convert tokens to `AbsoluteToken` for highlighting.
    pub fn to_abs_token(&self) -> Option<AbsoluteToken> {
        Some(AbsoluteToken {
            line: self.span.start.0,
            start: self.span.start.1,
            // NOTE: This will led to overflow if any token crosses any line.
            len: self.span.end.1 - self.span.start.1,
            ty: match_token_ty(&self.ty)?,
            mods: 0,
        })
    }
}

#[cfg(feature = "highlight")]
fn match_token_ty(ty: &TokenTy) -> Option<HighlightKind> {
    match ty {
        TokenTy::Quote => Some(HighlightKind::Quote),

        TokenTy::LeftBlock
        | TokenTy::LeftBlockStar
        | TokenTy::RightBlock
        | TokenTy::LeftBlockEnd
        | TokenTy::LeftBlockAnchor
        | TokenTy::LeftBracket
        | TokenTy::LeftBracketAnchor
        | TokenTy::LeftBracketStar
        | TokenTy::RightBracket
        // save herizontol rules to `Operator`
        | TokenTy::TripleDash
        | TokenTy::TableColumn
        | TokenTy::TableColumnRight
        | TokenTy::TableColumnCenter
        | TokenTy::TableColumnTitle
        => Some(HighlightKind::Operator),

        TokenTy::LeftLink
        | TokenTy::RightLink
        | TokenTy::LeftLinkStar
        | TokenTy::Url
        | TokenTy::Email => Some(HighlightKind::Link),

        TokenTy::Heading => Some(HighlightKind::Heading),
        _ => None,
    }
}
