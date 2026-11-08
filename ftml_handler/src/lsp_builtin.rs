mod blocks;
mod inline_blocks;
mod inline_marks;
mod regex;

use blocks::Blocks;
use inline_blocks::InlineBlocks;
use inline_marks::InlineMarks;

use crate::{
    ftml_ast::{Diagnostic, ParseToken, Token},
    utils::{is_inline_block_token, is_inline_mark_token},
};

use ftml::parsing::Token as TokenTy;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
/// A private enum for different types of builtin AST nodes.
enum BuiltIn<'lsp> {
    InLineMarks(InlineMarks<'lsp>),
    InLineBlocks(InlineBlocks<'lsp>),
    Blocks(Blocks<'lsp>),
    Token(&'lsp Token),
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// An easy sub highlight for different sub languages.
pub(crate) enum BlockKind {
    #[default]
    Block,
    ModuleCss,
    Html,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
/// The whole LSP builin AST will store in this struct.
pub struct LspBuiltIn<'lsp> {
    tree: Vec<BuiltIn<'lsp>>,
    diagnostics: &'lsp Vec<Diagnostic>,
}

impl<'lsp> LspBuiltIn<'lsp> {
    #[must_use]
    pub fn visitor(parse_token: &'lsp ParseToken) -> Self {
        let mut tree = Vec::new();
        let tokens = &parse_token.tokens;
        let mut visitor_index = 0;

        while visitor_index < tokens.len() {
            let (node, used) = visit(&tokens[visitor_index..], &parse_token.diagnostics);
            tree.push(node);
            visitor_index += used;
        }

        Self {
            tree,
            diagnostics: &parse_token.diagnostics,
        }
    }
}

/// A separate visitor function.
fn visit<'lsp>(
    parse_token: &'lsp [Token],
    diagnostics: &'lsp [Diagnostic],
) -> (BuiltIn<'lsp>, usize) {
    if parse_token[0].ty == TokenTy::LeftBlock
        && let Some(blocks) = Blocks::from_parse_token(parse_token, diagnostics)
    {
        let used = blocks.token_len();
        return (BuiltIn::Blocks(blocks), used);
    }

    if is_inline_mark_token(parse_token[0].ty)
        && let Some(inline_marks) = InlineMarks::from_parse_token_slice(parse_token, diagnostics)
    {
        let used = inline_marks.content_len() + 2;
        return (BuiltIn::InLineMarks(inline_marks), used);
    }

    if is_inline_block_token(parse_token[0].ty)
        && let Some(inline_blocks) = InlineBlocks::from_parse_token_slice(parse_token, diagnostics)
    {
        let used = inline_blocks.content_len() + 2;
        return (BuiltIn::InLineBlocks(inline_blocks), used);
    }
    (BuiltIn::Token(&parse_token[0]), 1)
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
/// This enum is set for different parameter expressions types,
/// which will NOT store as structs' fields.
enum ParamKind {
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
    fn match_param_kind(token_params: &[&Token]) -> Self {
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
/// An easy sub highlight in `Blocks` and `InlineBlocks` tokens.
enum ParamKeys {
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
    /// ""
    Quote,
    /// "string inside quote"
    Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Param<'lsp> {
    token: &'lsp Token,
    ty: ParamKeys,
}

impl<'lsp> Param<'lsp> {
    /// Identify attributes for `Blocks` and `InlineBlocks`.
    #[must_use]
    fn from_tokens(tokens: Vec<&'lsp Token>, param_kind: &ParamKind) -> Vec<Self> {
        match param_kind {
            ParamKind::Single => tokens
                .into_iter()
                .map(|token| Self {
                    token,
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
                    Self { token, ty }
                })
                .collect(),

            ParamKind::Expr => {
                let mut is_in_quote = false;
                tokens
                    .into_iter()
                    .enumerate()
                    .map(|(i, token)| {
                        let ty = match (i, token.ty) {
                            (0, _) => ParamKeys::BlockName,
                            (_, TokenTy::DoubleQuote) => {
                                is_in_quote = !is_in_quote;
                                ParamKeys::Quote
                            }
                            _ if is_in_quote => ParamKeys::Value,
                            (_, TokenTy::Whitespace) => ParamKeys::Whitespace,
                            (_, TokenTy::Pipe) => ParamKeys::Pipe,
                            (_, TokenTy::Equals) => ParamKeys::Equals,
                            _ => ParamKeys::BlockAttr,
                        };
                        Self { token, ty }
                    })
                    .collect()
            }
        }
    }
}
