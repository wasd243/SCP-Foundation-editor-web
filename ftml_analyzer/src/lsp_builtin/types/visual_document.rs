use serde::Serialize;

use crate::{
    Span,
    lsp_builtin::{BlockKind, BuiltIn},
};

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct VirtualDocument {
    pub lang: BlockKind, // only Html or ModuleCss ever gets here
    pub source: String,
}

impl VirtualDocument {
    #[must_use]
    pub fn new(lang: BlockKind, source: &str, tree: &[BuiltIn<'_>]) -> Option<Self> {
        let mut spans = Vec::new();

        collect_spans(tree, &lang, &mut spans);

        if spans.is_empty() {
            return None;
        }

        Some(Self {
            source: mask_lines(source, &spans),
            lang,
        })
    }
}

fn collect_spans(tree: &[BuiltIn<'_>], kind: &BlockKind, out: &mut Vec<Span>) {
    for node in tree {
        if let BuiltIn::Blocks(b) = node {
            if b.block_ty == *kind {
                out.extend(b.block_sub_lang_span);
            }

            collect_spans(&b.block_contents, kind, out);
        }
    }
}

fn mask_lines(source: &str, spans: &[Span]) -> String {
    let mut out = String::with_capacity(source.len());
    for (i, line) in source.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }

        let i = u32::try_from(i).unwrap_or(u32::MAX);

        if spans.iter().any(|s| i >= s.start.0 && i <= s.end.0) {
            out.push_str(line);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_builtin_ast, parse_token};

    const CSS_BLOCK: &str =
        "other text\n[[module css]]\n.cls {\ncolor: red;\n}\n[[/module]]\nOther text";

    #[test]
    fn check_css_visual_doc() {
        let token = parse_token(CSS_BLOCK);
        let tree = &get_builtin_ast(&token).tree;
        let css_visual_doc = VirtualDocument::new(BlockKind::ModuleCss, CSS_BLOCK, tree).unwrap();

        let expect = VirtualDocument {
            source: "\n\n.cls {\ncolor: red;\n}\n\n".to_owned(),
            lang: BlockKind::ModuleCss,
        };

        assert_eq!(css_visual_doc, expect);
    }
}
