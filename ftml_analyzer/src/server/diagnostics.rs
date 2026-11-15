use anyhow::Result;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    Diagnostic, PublishDiagnosticsParams, Uri,
    notification::{Notification as _, PublishDiagnostics},
};

use crate::{
    get_builtin_ast,
    lsp_builtin::{BlockKind, types::VirtualDocument},
    parse_token,
    server::{Buffers, web_service::WebService},
};

/// Push diagnostics to IDEs.
pub(super) fn push(
    connection: &Connection,
    buffers: &mut Buffers,
    uri: Uri,
    web: &mut WebService,
) -> Result<()> {
    let Some(source) = buffers.get(&uri) else {
        return manage_diagnostics(connection, uri, Vec::new());
    };

    let token = parse_token(source);
    let builtin = get_builtin_ast(&token);
    let mut diagnostics: Vec<Diagnostic> = token.to_diagnostics();

    for (lang, kind) in [("html", BlockKind::Html), ("css", BlockKind::ModuleCss)] {
        let Some(document) = VirtualDocument::new(kind, source, &builtin.tree) else {
            continue;
        };
        diagnostics.extend(web.diagnose(lang, &document.source)?);
    }

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
