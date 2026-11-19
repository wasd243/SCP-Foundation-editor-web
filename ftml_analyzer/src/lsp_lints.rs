//! This module is set for linting unnecessary code or incorrect.

mod empty_include;
mod unwrapped_fallbacks;

use lsp_types::{
    Diagnostic as ServerDiagnostic, DiagnosticSeverity, NumberOrString, Position,
    Range as ServerRange,
};
use serde::Serialize;

use crate::{Span, ftml_ast::Diagnostic, ftml_ast::Token};

#[derive(Debug, Serialize)]
pub struct LspLints {
    pub lints: Vec<LspLint>,
}

impl LspLints {
    pub fn linting(tokens: &[Token], diagnostics: &[Diagnostic]) -> Self {
        let mut lints = Vec::new();

        // Add lints here
        lints.extend(unwrapped_fallbacks::check(diagnostics));
        lints.extend(empty_include::check(tokens));

        Self { lints }
    }

    pub fn into_diagnostics(self) -> Vec<ServerDiagnostic> {
        self.lints
            .into_iter()
            .map(|d| ServerDiagnostic {
                range: ServerRange::new(
                    Position::new(d.span.start.0, d.span.start.1),
                    Position::new(d.span.end.0, d.span.end.1),
                ),
                severity: Some(d.ty.to_severity()),
                code: Some(NumberOrString::String(d.name.to_owned())),
                source: Some("ftml".to_owned()),
                message: d.msg,
                ..Default::default()
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
pub struct LspLint {
    name: &'static str,
    span: Span,
    msg: String,
    ty: LintKind,
}

impl LspLint {
    pub fn new(name: &'static str, span: Span, msg: String, ty: LintKind) -> Self {
        Self {
            name,
            span,
            msg,
            ty,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum LintKind {
    Correctness,
    Hint,
}

impl LintKind {
    fn to_severity(&self) -> DiagnosticSeverity {
        match self {
            Self::Correctness => DiagnosticSeverity::ERROR,
            Self::Hint => DiagnosticSeverity::HINT,
        }
    }
}
