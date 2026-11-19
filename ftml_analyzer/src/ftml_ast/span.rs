use std::ops::Range;

use line_index::{LineCol, LineIndex, TextSize, WideEncoding};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: (u32, u32),
    pub end: (u32, u32),
}

/// Convert `Range<usize>` to `Span`.
pub(super) fn from_index_to_span(index: &LineIndex, range: Range<usize>) -> Option<Span> {
    Some(Span {
        start: from_utf8_to_pos(index, range.start)?,
        end: from_utf8_to_pos(index, range.end)?,
    })
}

/// Convert utf-8 to line col positions.
pub(super) fn from_utf8_to_pos(index: &LineIndex, byte_off: usize) -> Option<(u32, u32)> {
    let line_col: LineCol = index.try_line_col(TextSize::new(byte_off as u32))?;
    let wide = index.to_wide(WideEncoding::Utf16, line_col)?;
    Some((wide.line, wide.col))
}
