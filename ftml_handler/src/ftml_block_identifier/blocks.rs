use crate::ftml_ast::{IncludeFoldMark, ParseToken, SubLangMark};
use crate::ftml_block_identifier::{
    IDENTIFIER, RIGHT_BLOCK,
    regex::*,
    utils::{check_end, is_open},
};

impl ParseToken {
    /// mark include blocks starts and ends
    pub(super) fn mark_includes(&mut self, index: usize) {
        if !is_open(self, index) {
            return;
        }

        let Some(end_idx) = self.find_include_end_(index) else {
            return;
        };

        if let Some(start_token) = self.tokens.get_mut(index) {
            start_token.include_fold_mark = Some(IncludeFoldMark::IncludeStart);
        }
        if let Some(end_token) = self.tokens.get_mut(end_idx) {
            end_token.include_fold_mark = Some(IncludeFoldMark::IncludeEnd);
        }
    }

    /// mark CSS modules
    pub(super) fn mark_module_css(&mut self, index: usize) {
        if !is_open(self, index) {
            return;
        }

        let mut value_iter = self.value_iter(index);

        let Some((_, token)) = value_iter.next() else {
            return;
        };

        if token.ty != IDENTIFIER || !MODULE.is_match(token.slice.as_str()) {
            return;
        }

        let Some((_, token)) = value_iter.next() else {
            return;
        };

        if token.ty != IDENTIFIER || !CSS.is_match(token.slice.as_str()) {
            return;
        }

        let Some((_, token)) = value_iter.next() else {
            return;
        };

        if token.ty != RIGHT_BLOCK {
            return;
        }

        let mut css_idx = Vec::new();
        for (i, _) in value_iter {
            if check_end(self, i, &MODULE) {
                break;
            }
            css_idx.push(i);
        }

        for i in css_idx {
            if let Some(css_token) = self.tokens.get_mut(i) {
                css_token.sub_lang_mark = Some(SubLangMark::Css);
            }
        }
    }

    /// Identify html blocks
    pub(super) fn mark_html(&mut self, index: usize) {
        if !is_open(self, index) {
            return;
        }

        let mut value_iter = self.value_iter(index);

        let Some((_, token)) = value_iter.next() else {
            return;
        };

        if token.ty != IDENTIFIER || !HTML.is_match(token.slice.as_str()) {
            return;
        }

        let Some((_, token)) = value_iter.next() else {
            return;
        };

        if token.ty != RIGHT_BLOCK {
            for _ in 0..self.tokens.len() {
                let Some((_, token)) = value_iter.next() else {
                    return;
                };

                if token.ty == RIGHT_BLOCK {
                    break;
                }
            }
        }

        let mut html_idx = Vec::new();
        for (i, _) in value_iter {
            if check_end(self, i, &HTML) {
                break;
            }
            html_idx.push(i);
        }

        for i in html_idx {
            if let Some(html_token) = self.tokens.get_mut(i) {
                html_token.sub_lang_mark = Some(SubLangMark::Html);
            }
        }
    }

    fn find_include_end_(&self, index: usize) -> Option<usize> {
        let mut value_iter = self.value_iter(index);

        let (_, token) = value_iter.next()?;
        if token.ty != IDENTIFIER || !INCLUDE.is_match(token.slice.as_str()) {
            return None;
        }

        value_iter.find_map(|(idx, candidate)| (candidate.ty == RIGHT_BLOCK).then_some(idx))
    }
}
