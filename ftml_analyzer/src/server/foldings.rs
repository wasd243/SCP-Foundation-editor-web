use lsp_types::{FoldingRange, FoldingRangeKind};

use crate::{lsp_builtin::BuiltIn, server::COLLAPSED_TEXT};

pub(super) fn fold(tree: Vec<BuiltIn<'_>>) -> Vec<FoldingRange> {
    let mut folding_ranges = Vec::new();
    for node in tree {
        match node {
            BuiltIn::InLineBlocks(b) => {
                let (open, close) = (b.start_block.span, b.end_block.span);

                folding_ranges.push(FoldingRange {
                    start_line: b.span.start.0,
                    end_line: b.span.end.0,
                    start_character: Some(open.start.1 + open.end.1),
                    end_character: Some(close.start.1 + close.end.1),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: Some(COLLAPSED_TEXT.to_owned()),
                });
            }
            BuiltIn::Blocks(b) => {
                if !b.block_contents.is_empty() {
                    folding_ranges.extend(fold(b.block_contents));
                }

                let tags = &b.starts_and_ends_of_the_block;

                let (open, close) = (tags[0].span, tags[3].span);

                folding_ranges.push(FoldingRange {
                    start_line: b.span.start.0,
                    end_line: b.span.end.0,
                    start_character: Some(open.start.1 + open.end.1),
                    end_character: Some(close.start.1 + close.end.1),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: Some(COLLAPSED_TEXT.to_owned()),
                });
            }
            _ => {}
        };
    }
    folding_ranges
}
