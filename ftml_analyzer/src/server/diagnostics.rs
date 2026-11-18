use anyhow::Result;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    Diagnostic, PublishDiagnosticsParams, Uri,
    notification::{Notification as _, PublishDiagnostics},
};

use crate::{
    LspLints,
    ftml_ast::ParseToken,
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

    let parse_token = parse_token(source);
    let builtin = get_builtin_ast(&parse_token);
    let mut diagnostics = build_diags(&parse_token);

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

fn build_diags(parse_token: &ParseToken) -> Vec<Diagnostic> {
    let lint_diags = LspLints::linting(
        parse_token.tokens.as_slice(),
        parse_token.diagnostics.as_slice(),
    )
    .into_diagnostics();

    let mut diagnostics: Vec<Diagnostic> = parse_token.to_diagnostics();
    diagnostics.extend(lint_diags);

    diagnostics
}

#[cfg(test)]
mod tests {
    use crate::{parse_token, server::diagnostics::build_diags};

    const SRC: &str = "{{[[include]]}}\n{{[[include]]}}\n";

    #[test]
    fn diagnostics_are_linted_correct() {
        let parse_token = parse_token(SRC);
        let diags = build_diags(&parse_token);
        dbg!(diags);
    }
}
