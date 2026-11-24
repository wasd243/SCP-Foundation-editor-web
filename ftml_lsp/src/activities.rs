mod completion;
mod diagnostic;
mod folding;
mod macros;
#[cfg(feature = "highlight")]
mod semantic;

use completion::completion;
use folding::folding_range_request;
#[cfg(feature = "highlight")]
use lsp_types::request::SemanticTokensFullRequest;
#[cfg(feature = "highlight")]
use semantic::semantic_full_request;

pub(crate) use macros::get_source;

use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    },
    request::{Completion, FoldingRangeRequest, Request as _},
};

use crate::activities::diagnostic::manage_diagnostics;

use super::Buffers;

pub(super) fn main_loop(connection: Connection) -> Result<()> {
    let mut buffers = Buffers::new();
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let response = match_requests(req, &buffers);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(n) => match_notifications(&connection, n, &mut buffers)?,
            Message::Response(_) => {}
        }
    }
    Ok(())
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
            diagnostic::push(connection, buffers, uri)?;
        }
        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = n.extract(DidChangeTextDocument::METHOD)?;
            let uri = p.text_document.uri;
            // Full sync: the last change holds the whole document text
            if let Some(change) = p.content_changes.into_iter().last() {
                buffers.insert(uri.clone(), change.text);
            }
            diagnostic::push(connection, buffers, uri)?;
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

/// Match difference requests from IDEs.
fn match_requests(req: Request, buffers: &Buffers) -> Response {
    match req.method.as_str() {
        FoldingRangeRequest::METHOD => folding_range_request(req, buffers),
        #[cfg(feature = "highlight")]
        SemanticTokensFullRequest::METHOD => semantic_full_request(req, buffers),
        Completion::METHOD => completion(req, buffers),
        _ => Response::new_err(
            req.id,
            lsp_server::ErrorCode::MethodNotFound as i32,
            format!("The {} is currently not supported.", req.method),
        ),
    }
}
