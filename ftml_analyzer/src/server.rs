#![allow(clippy::mutable_key_type)]

mod completions;
mod diagnostics;
mod foldings;
mod web_service;

use diagnostics::{manage_diagnostics, push as push_diagnostics};

use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, FoldingRangeParams,
    FoldingRangeProviderCapability, HoverParams, HoverProviderCapability, Position, SemanticToken,
    SemanticTokens, SemanticTokensParams, SemanticTokensResult, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    },
    request::{
        Completion, FoldingRangeRequest, HoverRequest, Request as _, SemanticTokensFullRequest,
    },
};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::{
    ftml_ast::ParseToken,
    get_builtin_ast,
    lsp_builtin::{BlockKind, BuiltIn, types::VirtualDocument},
    lsp_highlight::{
        AbsoluteSpanTree,
        types::{TYPES, semantic_tokens_capability},
    },
    parse_token,
    server::foldings::fold,
    utils::highlight::get_source,
};
use web_service::WebService;

const COLLAPSED_TEXT: &str = "...";

pub static SERVER_CAPABILITIES: LazyLock<ServerCapabilities> =
    LazyLock::new(|| ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        semantic_tokens_provider: Some(semantic_tokens_capability()),
        completion_provider: Some(CompletionOptions::default()),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        ..ServerCapabilities::default()
    });

pub type Buffers = HashMap<Uri, String>;

pub fn main_loop(connection: Connection) -> Result<()> {
    let mut buffers = HashMap::new();
    let mut web = WebService::new()?;
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let response = match_request(req, &buffers, &mut web);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(n) => {
                match_notifications(&connection, n, &mut buffers, &mut web)?
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

fn match_request(req: Request, buffers: &Buffers, web: &mut WebService) -> Response {
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

            let token = parse_token(source);
            let builtin = get_builtin_ast(&token);

            let mut absolute_tokens = decode_semantic_tokens(
                AbsoluteSpanTree::from_builtin_tree(builtin.tree).into_semantic_tokens(),
            );
            if let Err(error) = append_embedded_tokens(web, source, &token, &mut absolute_tokens) {
                return Response::new_err(
                    id,
                    lsp_server::ErrorCode::InternalError as i32,
                    error.to_string(),
                );
            }
            let tokens = encode_semantic_tokens(absolute_tokens);

            Response::new_ok(
                id,
                SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: tokens,
                }),
            )
        }
        Completion::METHOD => {
            let (id, params): (_, CompletionParams) = match req.clone().extract(Completion::METHOD)
            {
                Ok(value) => value,
                Err(error) => {
                    return Response::new_err(
                        req.id,
                        lsp_server::ErrorCode::InvalidParams as i32,
                        error.to_string(),
                    );
                }
            };
            let Some(source) = buffers.get(&params.text_document_position.text_document.uri) else {
                return Response::new_err(
                    id,
                    lsp_server::ErrorCode::InvalidParams as i32,
                    "The requested document is not open.".to_owned(),
                );
            };
            let token = parse_token(source);
            let builtin = get_builtin_ast(&token);
            let Some(lang) =
                embedded_language(&builtin.tree, params.text_document_position.position)
            else {
                return Response::new_ok(id, None::<CompletionResponse>);
            };
            let Some(document) = virtual_source(lang, source, &builtin.tree) else {
                return Response::new_ok(id, None::<CompletionResponse>);
            };
            match web.complete(lang, &document, params.text_document_position.position) {
                Ok(list) => Response::new_ok(id, Some(CompletionResponse::List(list))),
                Err(error) => Response::new_err(
                    id,
                    lsp_server::ErrorCode::InternalError as i32,
                    error.to_string(),
                ),
            }
        }
        HoverRequest::METHOD => {
            let (id, params): (_, HoverParams) = match req.clone().extract(HoverRequest::METHOD) {
                Ok(value) => value,
                Err(error) => {
                    return Response::new_err(
                        req.id,
                        lsp_server::ErrorCode::InvalidParams as i32,
                        error.to_string(),
                    );
                }
            };
            let Some(source) = buffers.get(&params.text_document_position_params.text_document.uri)
            else {
                return Response::new_err(
                    id,
                    lsp_server::ErrorCode::InvalidParams as i32,
                    "The requested document is not open.".to_owned(),
                );
            };
            let token = parse_token(source);
            let builtin = get_builtin_ast(&token);
            let Some(lang) =
                embedded_language(&builtin.tree, params.text_document_position_params.position)
            else {
                return Response::new_ok(id, None::<lsp_types::Hover>);
            };
            let Some(document) = virtual_source(lang, source, &builtin.tree) else {
                return Response::new_ok(id, None::<lsp_types::Hover>);
            };
            match web.hover(
                lang,
                &document,
                params.text_document_position_params.position,
            ) {
                Ok(hover) => Response::new_ok(id, hover),
                Err(error) => Response::new_err(
                    id,
                    lsp_server::ErrorCode::InternalError as i32,
                    error.to_string(),
                ),
            }
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
    web: &mut WebService,
) -> Result<()> {
    match n.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = n.extract(DidOpenTextDocument::METHOD)?;
            let uri = p.text_document.uri;
            buffers.insert(uri.clone(), p.text_document.text);
            push_diagnostics(connection, buffers, uri, web)?;
        }
        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = n.extract(DidChangeTextDocument::METHOD)?;
            let uri = p.text_document.uri;
            // Full sync: the last change holds the whole document text
            if let Some(change) = p.content_changes.into_iter().last() {
                buffers.insert(uri.clone(), change.text);
            }
            push_diagnostics(connection, buffers, uri, web)?;
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

fn embedded_language(tree: &[BuiltIn<'_>], position: Position) -> Option<&'static str> {
    for node in tree {
        let BuiltIn::Blocks(block) = node else {
            continue;
        };

        if let Some(span) = block.block_sub_lang_span
            && span_contains(span, position)
        {
            return match block.block_ty {
                BlockKind::Html => Some("html"),
                BlockKind::ModuleCss => Some("css"),
                BlockKind::Block => None,
            };
        }

        if let Some(lang) = embedded_language(&block.block_contents, position) {
            return Some(lang);
        }
    }

    None
}

fn span_contains(span: crate::Span, position: Position) -> bool {
    (span.start.0..=span.end.0).contains(&position.line)
        && (position.line != span.start.0 || position.character >= span.start.1)
        && (position.line != span.end.0 || position.character <= span.end.1)
}

fn virtual_source(lang: &str, source: &str, tree: &[BuiltIn<'_>]) -> Option<String> {
    let kind = match lang {
        "html" => BlockKind::Html,
        "css" => BlockKind::ModuleCss,
        _ => return None,
    };
    VirtualDocument::new(kind, source, tree).map(|document| document.source)
}

fn append_embedded_tokens(
    web: &mut WebService,
    source: &str,
    token: &ParseToken,
    tokens: &mut Vec<(u32, u32, u32, u32, u32)>,
) -> Result<()> {
    let builtin = get_builtin_ast(token);
    for (lang, kind) in [("html", BlockKind::Html), ("css", BlockKind::ModuleCss)] {
        let Some(document) = VirtualDocument::new(kind, source, &builtin.tree) else {
            continue;
        };
        for token in web.tokens(lang, &document.source)? {
            let Some(token_type) = TYPES
                .iter()
                .position(|item| item.as_str() == token.kind)
                .map(|index| index as u32)
            else {
                continue;
            };
            tokens.push((token.line, token.character, token.length, token_type, 0));
        }
    }
    Ok(())
}

fn decode_semantic_tokens(tokens: Vec<SemanticToken>) -> Vec<(u32, u32, u32, u32, u32)> {
    let mut line = 0;
    let mut start = 0;
    let mut absolute = Vec::with_capacity(tokens.len());

    for token in tokens {
        line += token.delta_line;
        start = if token.delta_line == 0 {
            start + token.delta_start
        } else {
            token.delta_start
        };
        absolute.push((
            line,
            start,
            token.length,
            token.token_type,
            token.token_modifiers_bitset,
        ));
    }

    absolute
}

fn encode_semantic_tokens(mut tokens: Vec<(u32, u32, u32, u32, u32)>) -> Vec<SemanticToken> {
    tokens.sort_unstable_by_key(|&(line, start, ..)| (line, start));
    let (mut previous_line, mut previous_start) = (0, 0);

    tokens
        .into_iter()
        .map(|(line, start, length, token_type, modifiers)| {
            let delta_line = line - previous_line;
            let delta_start = if delta_line == 0 {
                start - previous_start
            } else {
                start
            };
            previous_line = line;
            previous_start = start;
            SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type,
                token_modifiers_bitset: modifiers,
            }
        })
        .collect()
}
