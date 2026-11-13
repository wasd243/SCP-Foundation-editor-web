//! # LSP BuiltIn
//!
//! This module is set for building AST with span.
//! The AST is builtin only for frontend highlightings rather than parsing.

mod blocks;
mod comments;
mod inline_blocks;
mod inline_marks;
mod regex;
pub mod types;

use blocks::Blocks;
use inline_blocks::InlineBlocks;
use inline_marks::InlineMarks;
use types::*;

use crate::{
    ftml_ast::{Diagnostic, ParseToken, Token},
    lsp_builtin::comments::Comments,
    utils::{is_inline_block_token, is_inline_mark_token},
};

use ftml::parsing::Token as TokenTy;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
/// A private enum for different types of builtin AST nodes.
pub enum BuiltIn<'lsp> {
    InLineMarks(InlineMarks<'lsp>),
    InLineBlocks(InlineBlocks<'lsp>),
    Blocks(Blocks<'lsp>),
    Comments(Comments<'lsp>),
    Token(&'lsp Token),
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// An easy sub highlight for different sub languages.
pub enum BlockKind {
    #[default]
    Block,
    ModuleCss,
    Html,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
/// The whole LSP builin AST will store in this struct.
pub struct LspBuiltIn<'lsp> {
    pub tree: Vec<BuiltIn<'lsp>>,
    pub diagnostics: &'lsp Vec<Diagnostic>,
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

    /// Remove unnecessary `Whitespace`.
    pub fn sanitize_whitespace(&mut self) {
        self.tree
            .retain(|node| !matches!(node, BuiltIn::Token(t) if t.ty == TokenTy::Whitespace));
    }

    /// Build an `VisualDocument` for calling other LSPs on HTML/CSS embended languages highlights.
    pub fn build_visual_document(&self) {}
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

    if parse_token[0].ty == TokenTy::LeftComment
        && let Some(comments) = Comments::from_parse_token_slice(parse_token, diagnostics)
    {
        let used = comments.content_len();
        return (BuiltIn::Comments(comments), used);
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
