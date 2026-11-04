mod inline_marks;

use inline_marks::InlineMarks;

use crate::ftml_ast::Diagnostic;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum BuiltIn<'lsp> {
    InLineMarks(InlineMarks<'lsp>),
}

#[derive(Debug, Serialize)]
pub struct LspBuiltIn<'lsp> {
    pub tree: Vec<BuiltIn<'lsp>>,
    pub diagnostics: Vec<Diagnostic>,
}
