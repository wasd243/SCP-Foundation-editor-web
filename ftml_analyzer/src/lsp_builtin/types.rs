use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::Token,
    lsp_highlight::{AbsoluteSpan, HighlightTy, ToAbsoluteSpan},
};

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
/// This enum is set for different parameter expressions types,
/// which will NOT store as structs' fields.
pub enum ParamKind {
    /// Normal blocks like `[[note]][[/note]]` `[[footnote]] [[/footnote]]`
    /// Also includes `[[div]] [[/div]]` `[[module]] [[/module]]`.
    /// This value means all `Blocks` with a single parameter.
    #[default]
    Single,
    /// Blocks have attrs, which includes `[[module css]]`.
    Attr,
    /// Expressions within `=`.
    Expr,
}

impl ParamKind {
    /// Identify parameter kinds from token slice.
    #[must_use]
    pub fn match_param_kind(token_params: &[&Token]) -> Self {
        let mut param_kind = Self::default();
        if token_params.len() > 1 {
            param_kind = Self::Attr;
            if token_params.iter().any(|t| matches!(t.ty, TokenTy::Equals)) {
                param_kind = Self::Expr;
            }
        }
        param_kind
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
/// An easy sub highlight in `Blocks` and `InlineBlocks` tokens.
pub enum ParamKeys {
    /// Block name like `footnote`
    BlockName,
    /// Block attributes like:
    /// ```ftml,ignore
    /// [[module css]]
    ///          ^^^
    /// ```
    BlockAttr,
    /// For " " Whitespace
    Whitespace,
    /// |
    Pipe,
    /// =
    Equals,
    /// "string inside quote"
    String,
    /// Value after `=` equals
    Value,
    /// :
    Colon,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub struct Param {
    pub span: Span,
    pub ty: ParamKeys,
}

impl Param {
    /// Identify attributes for `Blocks` and `InlineBlocks`.
    #[must_use]
    pub fn from_tokens(tokens: Vec<&Token>, param_kind: &ParamKind) -> Vec<Self> {
        match param_kind {
            ParamKind::Single => tokens
                .into_iter()
                .map(|token| Self {
                    span: token.span,
                    ty: ParamKeys::BlockName,
                })
                .collect(),

            ParamKind::Attr => tokens
                .into_iter()
                .enumerate()
                .map(|(i, token)| {
                    let ty = match (i, token.ty) {
                        (0, _) => ParamKeys::BlockName,
                        (_, TokenTy::Whitespace) => ParamKeys::Whitespace,
                        _ => ParamKeys::BlockAttr,
                    };
                    Self {
                        span: token.span,
                        ty,
                    }
                })
                .collect(),

            ParamKind::Expr => {
                let mut is_in_quote = false;
                let mut after_equals = false;
                tokens
                    .into_iter()
                    .enumerate()
                    .map(|(i, token)| {
                        let ty = match (i, token.ty) {
                            (0, _) => ParamKeys::BlockName,
                            (_, TokenTy::DoubleQuote) => {
                                is_in_quote = !is_in_quote;
                                ParamKeys::String
                            }
                            _ if is_in_quote => ParamKeys::String,
                            (_, TokenTy::Whitespace) => {
                                after_equals = false;
                                ParamKeys::Whitespace
                            }
                            (_, TokenTy::Pipe) => {
                                after_equals = false;
                                ParamKeys::Pipe
                            }
                            (_, TokenTy::Equals) => {
                                after_equals = true;
                                ParamKeys::Equals
                            }
                            _ if after_equals => ParamKeys::Value,
                            (_, TokenTy::Colon) => ParamKeys::Colon,
                            _ => ParamKeys::BlockAttr,
                        };
                        Self {
                            span: token.span,
                            ty,
                        }
                    })
                    .collect()
            }
        }
    }
}

impl ToAbsoluteSpan for Param {
    fn to_abs_span(&self) -> Option<AbsoluteSpan> {
        Some(AbsoluteSpan {
            line: self.span.start.0,
            start: self.span.start.1,
            len: self.span.end.1 - self.span.start.1,
            ty: match_param_keys(&self.ty)?,
            // TODO: mods is a placeholder
            mods: 0,
        })
    }
}

fn match_param_keys(ty: &ParamKeys) -> Option<HighlightTy> {
    match ty {
        ParamKeys::BlockName => Some(HighlightTy::KeyWord),
        ParamKeys::BlockAttr => Some(HighlightTy::Attr),
        ParamKeys::Equals => Some(HighlightTy::Equals),
        ParamKeys::Pipe => Some(HighlightTy::Pipe),
        ParamKeys::String => Some(HighlightTy::String),
        ParamKeys::Value => Some(HighlightTy::Value),
        ParamKeys::Colon => Some(HighlightTy::PathBuffer),
        // NOTE: Skip white spaces as `None`.
        ParamKeys::Whitespace => None,
    }
}
