use lsp_server::{Request, Response};
use lsp_types::{
    SemanticTokens, SemanticTokensParams, SemanticTokensResult,
    request::{Request as _, SemanticTokensFullRequest},
};

use crate::Buffers;

use super::get_source;

/// Get semantic tokens from source, NOT enabled by default.
pub(super) fn semantic_full_request(req: Request, buffers: &Buffers) -> Response {
    get_source!(
        req,
        buffers,
        SemanticTokensParams,
        SemanticTokensFullRequest::METHOD,
        id,
        source
    );

    let builtin = ftml_analyzer::get_ast(source);
    let tokens = builtin.to_semantic();
    Response::new_ok(
        id,
        SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        }),
    )
}
