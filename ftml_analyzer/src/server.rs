#![allow(clippy::mutable_key_type)]

use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    FoldingRangeProviderCapability, PublishDiagnosticsParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
        PublishDiagnostics,
    },
    request::{FoldingRangeRequest, Request as _},
};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::parse_token;

pub static SERVER_CAPABILITIES: LazyLock<ServerCapabilities> =
    LazyLock::new(|| ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        ..ServerCapabilities::default()
    });

pub type Buffers = HashMap<Uri, String>;

pub fn main_loop(connection: Connection) -> Result<()> {
    let mut buffers = HashMap::new();
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let response = match_request(req, &buffers);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(n) => match_notifications(&connection, n, &mut buffers)?,
            Message::Response(_) => {}
        }
    }
    Ok(())
}

fn match_request(req: Request, _buffers: &Buffers) -> Response {
    match req.method.as_str() {
        FoldingRangeRequest::METHOD => {
            Response::new_ok(req.id, Vec::<lsp_types::FoldingRange>::new())
        }
        _ => Response::new_err(
            req.id,
            lsp_server::ErrorCode::MethodNotFound as i32,
            format!("The {} is currently not supported.", req.method),
        ),
    }
}

fn match_notifications(
    connection: &Connection,
    n: Notification,
    buffers: &mut Buffers,
) -> Result<()> {
    match n.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = n.extract(DidOpenTextDocument::METHOD)?;
            let uri = p.text_document.uri;
            buffers.insert(uri.clone(), p.text_document.text);
            push(connection, buffers, uri)?;
        }
        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = n.extract(DidChangeTextDocument::METHOD)?;
            let uri = p.text_document.uri;
            // Full sync: the last change holds the whole document text
            if let Some(change) = p.content_changes.into_iter().last() {
                buffers.insert(uri.clone(), change.text);
            }
            push(connection, buffers, uri)?;
        }
        DidCloseTextDocument::METHOD => {
            let p: DidCloseTextDocumentParams = n.extract(DidCloseTextDocument::METHOD)?;
            buffers.remove(&p.text_document.uri);
            manage_diagnostics(connection, p.text_document.uri, Vec::new())?; // clear on close
        }
        _ => {}
    }
    Ok(())
}

/// Push diagnostics to IDEs.
fn push(connection: &Connection, buffers: &mut Buffers, uri: Uri) -> Result<()> {
    let diags = buffers
        .get(&uri)
        .map(|src| parse_token(src).to_diagnostics())
        .unwrap_or_default();
    manage_diagnostics(connection, uri, diags)
}

fn manage_diagnostics(
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
