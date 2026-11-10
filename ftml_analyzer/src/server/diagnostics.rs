use anyhow::Result;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    PublishDiagnosticsParams, Uri,
    notification::{Notification as _, PublishDiagnostics},
};

use crate::{parse_token, server::Buffers};

/// Push diagnostics to IDEs.
pub(super) fn push(connection: &Connection, buffers: &mut Buffers, uri: Uri) -> Result<()> {
    let diags = buffers
        .get(&uri)
        .map(|src| parse_token(src).to_diagnostics())
        .unwrap_or_default();
    manage_diagnostics(connection, uri, diags)
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
