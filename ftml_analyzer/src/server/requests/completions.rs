mod types;
use types::BLOCKS;

use lsp_server::{Request, Response};
use lsp_types::{
    CompletionParams, CompletionResponse, Position,
    request::{Completion, Request as _},
};

use crate::{
    get_builtin_ast, parse_token,
    server::{Buffers, web_service::WebService},
    utils::highlight::get_source,
};

use super::{embedded_language, virtual_source};

pub(crate) fn completion(req: Request, buffers: &Buffers, web: &mut WebService) -> Response {
    get_source!(
        req,
        buffers,
        CompletionParams,
        Completion::METHOD,
        id,
        source,
        params
    );

    let token = parse_token(source);
    let builtin = get_builtin_ast(&token);
    let pos: Position = params.text_document_position.position;

    let Some(lang) = embedded_language(&builtin.tree, pos) else {
        let result = ftml_completion(source, pos);
        return Response::new_ok(id, result);
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

fn ftml_completion(source: &str, pos: Position) -> Option<CompletionResponse> {
    let before = before(source, pos);
    let ctx = Ctx::new(before)?;

    let items = match ctx {
        Ctx::BlockName => BLOCKS.iter().map(|b| b.to_completion_item()).collect(),
        // TODO: finish attr name completions
        Ctx::AttrName(_) => return None,
    };
    Some(CompletionResponse::Array(items))
}

#[derive(Debug)]
enum Ctx {
    /// div/module/...
    BlockName,
    /// [[div
    AttrName(String),
}

impl Ctx {
    fn new(before: &str) -> Option<Self> {
        let open = before.rfind("[[")?;
        let tail = &before[open + 2..];

        if tail.contains("]]") {
            return None;
        }

        let tail = tail.trim_start_matches(['*', '/']);
        match tail.split_once(char::is_whitespace) {
            None => Some(Self::BlockName),
            Some((name, _rest)) => Some(Ctx::AttrName(name.to_owned())),
        }
    }
}

fn before(source: &str, pos: Position) -> &str {
    let mut offset = 0usize;
    for (i, line) in source.split('\n').enumerate() {
        if i == pos.line as usize {
            let mut units = 0u32;
            for (byte_idx, ch) in line.char_indices() {
                if units >= pos.character {
                    return &source[..offset + byte_idx];
                }
                units += ch.len_utf16() as u32;
            }
            return &source[..offset + line.len()];
        }
        offset += line.len() + 1; // +1 is '\n'
    }
    source
}
