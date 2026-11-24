use ftml_analyzer::nodes::{BuiltIn, Node};
use lsp_server::{Request, Response};
use lsp_types::{
    FoldingRange, FoldingRangeKind, FoldingRangeParams,
    request::{FoldingRangeRequest, Request as _},
};

use crate::{Buffers, activities::get_source};

pub(super) fn folding_range_request(req: Request, buffers: &Buffers) -> Response {
    get_source!(
        req,
        buffers,
        FoldingRangeParams,
        FoldingRangeRequest::METHOD,
        id,
        source
    );

    let builtin = ftml_analyzer::get_ast(source);
    let folding_ranges = fold(builtin);

    Response::new_ok(id, folding_ranges)
}

macro_rules! fold {
    ($node:expr, $ty:expr) => {
        ::lsp_types::FoldingRange {
            start_line: $node.span.start.0,
            start_character: Some($node.span.start.1),
            end_line: $node.span.end.0,
            end_character: Some($node.span.end.1),
            kind: Some($ty),
            collapsed_text: Some($crate::COLLAPSED_TEXT.to_string()),
        }
    };
}

/// Based on builtin, get the folding ranges.
fn fold(tree: BuiltIn) -> Vec<FoldingRange> {
    tree.nodes
        .iter()
        .filter_map(|n| match n {
            Node::Block(b) => Some(fold!(b, FoldingRangeKind::Region)),
            Node::Comment(c) => Some(fold!(c, FoldingRangeKind::Comment)),
            _ => None,
        })
        .collect()
}
