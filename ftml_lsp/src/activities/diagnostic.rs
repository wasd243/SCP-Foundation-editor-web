use anyhow::Result;
use ftml_analyzer::{get_ast, nodes::BuiltIn};
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, Position, PublishDiagnosticsParams, Range, Uri,
    notification::{Notification as _, PublishDiagnostics},
};

use crate::Buffers;

pub(super) fn push(connection: &Connection, buffers: &mut Buffers, uri: Uri) -> Result<()> {
    let Some(source) = buffers.get(&uri) else {
        return manage_diagnostics(connection, uri, Vec::new());
    };

    let ast = get_ast(source);
    let diagnostics = build_diags(ast);

    manage_diagnostics(connection, uri, diagnostics)
}

pub(super) fn manage_diagnostics(
    connection: &Connection,
    uri: Uri,
    diagnostics: Vec<lsp_types::Diagnostic>,
) -> Result<()> {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    let n = Notification::new(PublishDiagnostics::METHOD.to_string(), params);
    connection.sender.send(Message::Notification(n))?;
    Ok(())
}

// NOTE: lint system unimplemented temporarily, directly use `BuiltIn`'s diagnostics.
// move the `diagnostics` this time, do not clone.
fn build_diags(ast: BuiltIn) -> Vec<Diagnostic> {
    let diagnostics: Vec<ftml_analyzer::ftml_ast::Diagnostic> = ast.diagnostics;

    diagnostics
        .into_iter()
        .map(|d| Diagnostic {
            range: Range {
                start: Position::new(d.span.start.0, d.span.start.1),
                end: Position::new(d.span.end.0, d.span.end.1),
            },
            severity: Some(DiagnosticSeverity::WARNING),
            source: Some("ftml-analyzer".to_string()),
            message: d.msg,
            ..Default::default()
        })
        .collect()
}
