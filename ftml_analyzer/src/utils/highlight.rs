use crate::{
    lsp_builtin::BuiltIn,
    lsp_highlight::{AbsoluteSpan, ToAbsoluteSpan as _, ToAbsoluteSpans as _},
};

/// Color each nodes based on BuiltIn tree.
pub(crate) fn color_nodes(node: &BuiltIn<'_>, spans: &mut Vec<AbsoluteSpan>) {
    match node {
        BuiltIn::Token(t) => spans.extend(t.to_abs_span()),
        BuiltIn::InLineMarks(m) => spans.extend(m.to_abs_spans()),
        BuiltIn::InLineBlocks(b) => spans.extend(b.to_abs_spans()),
        BuiltIn::Blocks(b) => spans.extend(b.to_abs_spans()),
        BuiltIn::Comments(c) => spans.extend(c.to_abs_spans()),
    }
}

macro_rules! get_source {
    ($req:ident, $buffers:ident, $req_ty:ty, $method:expr, $id:ident, $source:ident) => {
        let ($id, params): (_, $req_ty) = $req.clone().extract($method).unwrap();

        let Some($source) = $buffers.get(&params.text_document.uri) else {
            return lsp_server::Response::new_err(
                $id,
                lsp_server::ErrorCode::InvalidParams as i32,
                "The requested document is not open.".to_owned(),
            );
        };
    };
}

pub(crate) use get_source;
