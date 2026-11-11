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
    }
}
