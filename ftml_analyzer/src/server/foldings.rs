use lsp_types::{FoldingRange, FoldingRangeKind};

use crate::{Span, lsp_builtin::BuiltIn, server::COLLAPSED_TEXT};

pub(super) fn fold(tree: Vec<BuiltIn<'_>>) -> Vec<FoldingRange> {
    let mut folding_ranges = Vec::new();
    for node in tree {
        match node {
            BuiltIn::InLineBlocks(b) => {
                folding_ranges.push(fold_region(b.span, b.start_block.span, b.end_block.span));
            }
            BuiltIn::Blocks(b) => {
                if !b.block_contents.is_empty() {
                    folding_ranges.extend(fold(b.block_contents));
                }
                let tags = &b.delimiters;
                let (open, close) = (tags[0].span, tags[3].span);

                folding_ranges.push(fold_region(b.span, open, close));
            }
            BuiltIn::Comments(c) => {
                folding_ranges.push(FoldingRange {
                    start_line: c.span.start.0,
                    end_line: c.span.end.0,
                    start_character: None,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Comment),
                    collapsed_text: Some(COLLAPSED_TEXT.to_owned()),
                });
            }
            _ => {}
        };
    }
    folding_ranges
}

fn fold_region(span: Span, open: Span, close: Span) -> FoldingRange {
    FoldingRange {
        start_line: span.start.0,
        end_line: span.end.0,
        start_character: Some(open.start.1 + open.end.1),
        end_character: Some(close.start.1 + close.end.1),
        kind: Some(FoldingRangeKind::Region),
        collapsed_text: Some(COLLAPSED_TEXT.to_owned()),
    }
}
