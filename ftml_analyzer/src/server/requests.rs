use lsp_server::{Request, Response};
use lsp_types::{
    CompletionParams, CompletionResponse, FoldingRangeParams, HoverParams, SemanticTokens,
    SemanticTokensParams, SemanticTokensResult,
    request::{
        Completion, FoldingRangeRequest, HoverRequest, Request as _, SemanticTokensFullRequest,
    },
};

use crate::{
    ftml_ast::ParseToken,
    get_builtin_ast,
    lsp_highlight::AbsoluteSpanTree,
    parse_token,
    server::{Buffers, fold, web_service::WebService},
    utils::highlight::get_source,
};

use super::{
    append_embedded_tokens, decode_semantic_tokens, embedded_language, encode_semantic_tokens,
    virtual_source,
};

pub(super) fn folding_range_request(req: Request, buffers: &Buffers) -> Response {
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

pub(super) fn semantic_full_request(
    req: Request,
    buffers: &Buffers,
    web: &mut WebService,
) -> Response {
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

pub(super) fn completion(req: Request, buffers: &Buffers, web: &mut WebService) -> Response {
    let (id, params): (_, CompletionParams) = match req.clone().extract(Completion::METHOD) {
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
    let Some(lang) = embedded_language(&builtin.tree, params.text_document_position.position)
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

pub(super) fn hover_request(req: Request, buffers: &Buffers, web: &mut WebService) -> Response {
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
    let Some(source) = buffers.get(&params.text_document_position_params.text_document.uri) else {
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
