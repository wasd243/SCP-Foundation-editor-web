mod folding;
mod macros;
#[cfg(feature = "highlight")]
mod semantic;

use folding::folding_range_request;
#[cfg(feature = "highlight")]
use semantic::semantic_full_request;

pub(crate) use macros::get_source;

use std::collections::HashMap;

use anyhow::Result;
use lsp_server::{Connection, Message, Request, Response};
use lsp_types::request::{
    Completion, FoldingRangeRequest, HoverRequest, Request as _, SemanticTokensFullRequest,
};

use super::Buffers;

pub(super) fn main_loop(connection: Connection) -> Result<()> {
    let mut buffers = HashMap::new();
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

/// Match difference requests from IDEs.
fn match_requests(req: Request, buffers: &Buffers) -> Response {
    match req.method.as_str() {
        FoldingRangeRequest::METHOD => folding_range_request(req, buffers),
        #[cfg(feature = "highlight")]
        SemanticTokensFullRequest::METHOD => semantic_full_request(req, buffers),
        Completion::METHOD => completion(req, buffers),
        HoverRequest::METHOD => hover_request(req, buffers),
        _ => Response::new_err(
            req.id,
            lsp_server::ErrorCode::MethodNotFound as i32,
            format!("The {} is currently not supported.", req.method),
        ),
    }
}
