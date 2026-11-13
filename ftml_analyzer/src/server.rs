#![allow(clippy::mutable_key_type)]

mod completions;
mod diagnostics;
mod foldings;

use diagnostics::{manage_diagnostics, push};

use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    FoldingRangeParams, FoldingRangeProviderCapability, SemanticToken, SemanticTokens,
    SemanticTokensParams, SemanticTokensResult, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Uri,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    },
    request::{FoldingRangeRequest, Request as _, SemanticTokensFullRequest},
};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::{
    ftml_ast::ParseToken, get_builtin_ast, lsp_highlight::types::semantic_tokens_capability, parse,
    parse_token, server::foldings::fold, utils::highlight::get_source,
};

const COLLAPSED_TEXT: &str = "...";

pub static SERVER_CAPABILITIES: LazyLock<ServerCapabilities> =
    LazyLock::new(|| ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        semantic_tokens_provider: Some(semantic_tokens_capability()),
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

fn match_request(req: Request, buffers: &Buffers) -> Response {
    match req.method.as_str() {
        FoldingRangeRequest::METHOD => {
            get_source!(
                req,
                buffers,
                FoldingRangeParams,
                FoldingRangeRequest::METHOD,
                id,
                source
            );

            let tokens: ParseToken = parse_token(source);
            let builtin_tree = get_builtin_ast(&tokens).tree;
            let folding_ranges = fold(builtin_tree);
            Response::new_ok(id, folding_ranges)
        }
        SemanticTokensFullRequest::METHOD => {
            get_source!(
                req,
                buffers,
                SemanticTokensParams,
                SemanticTokensFullRequest::METHOD,
                id,
                source
            );

            let tokens: Vec<SemanticToken> = parse(source);
            Response::new_ok(
                id,
                SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: tokens,
                }),
            )
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
