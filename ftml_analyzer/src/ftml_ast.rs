//! # FTML AST
//!
//! This module handles ftml's tokenstream and parse errors.
//! For the LSP, we only needs tokens and diagnostics, so the parse results are dropped.
//!
//! # Known Issues
//! It's difficult to resolve and fetch `[[include]]` included pages, so ftml produce false
//! postivies warnings on `[[include]]`, use `tokens.mute_ftml_fps()`
//! to remove those false postivies.

mod span;
mod token;

use ftml::parsing::{ExtractedToken, ParseErrorKind};
use ftml::prelude::*;
use line_index::LineIndex;
use serde::Serialize;
use std::sync::LazyLock;

pub use span::Span;
pub use token::Token;

use crate::ftml_ast::span::from_index_to_span;

#[derive(Serialize, Debug, Default)]
pub struct ParseToken {
    /// ftml tokens
    pub tokens: Vec<Token>,
    /// ftml parse warnings
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize, Debug)]
pub struct Diagnostic {
    pub msg: String,
    pub ty: ParseErrorKind,
    pub span: Span,
}

impl ParseToken {
    pub fn clean(&mut self) {
        use ftml::parsing::Token;

        self.tokens.retain(|token| {
            !matches!(
                token.ty,
                Token::InputStart | Token::InputEnd | Token::LineBreak | Token::ParagraphBreak
            )
        });
    }

    /// TODO: `ftml` currently produce false postivies diagnostics when everything offline
    /// and could not fetch included pages.
    pub fn mute_ftml_fps(&mut self) {
        self.diagnostics.sort_by_key(|d| d.span.start);
        let mut diagnostic_need_remove = vec![false; self.diagnostics.len()];

        for (i, d) in self.diagnostics.windows(3).enumerate() {
            if matches!(d[0].ty, ParseErrorKind::NoRulesMatch)
                && matches!(d[1].ty, ParseErrorKind::InvalidInclude)
                && matches!(d[2].ty, ParseErrorKind::NoRulesMatch)
            {
                diagnostic_need_remove[i..i + 3].fill(true);
            }
        }

        let mut idx = 0;
        self.diagnostics.retain(|_| {
            let keep = !diagnostic_need_remove[idx];
            idx += 1;
            keep
        });
    }
}

/// implementations for &str.
impl From<&str> for ParseToken {
    /// Parse new token from wikitext
    fn from(src: &str) -> Self {
        let wikitext = src.to_string();
        let mut parse_token = from_(wikitext);
        parse_token.clean();
        parse_token.mute_ftml_fps();

        parse_token
    }
}

/// implementations for String.
impl From<String> for ParseToken {
    fn from(src: String) -> Self {
        let mut parse_token = from_(src);
        parse_token.clean();
        parse_token.mute_ftml_fps();

        parse_token
    }
}

/// Inner function for `From` trait implementations.
fn from_(src: String) -> ParseToken {
    let index = LineIndex::new(&src);
    let (tokens, diagnostics) = get_ftml_token(&src, &index);
    let tokens = tokens
        .into_iter()
        .filter_map(|t| {
            let span = from_index_to_span(&index, t.span.clone())?;
            Some(Token::from_extracted_tokens_with_span(t, span))
        })
        .collect();

    ParseToken {
        tokens,
        diagnostics,
    }
}

static DEFAULT_SETTINGS: LazyLock<WikitextSettings> =
    LazyLock::new(|| WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot));

static DEFAULT_PAGE_INFO: LazyLock<PageInfo> = LazyLock::new(|| PageInfo {
    page: Default::default(),
    category: None,
    site: Default::default(),
    title: Default::default(),
    alt_title: None,
    score: ScoreValue::Integer(0),
    tags: vec![],
    language: Default::default(),
});

/// Tokenize without preprocessing, so spans match the editor text.
/// Spans are converted from UTF-8 byte offsets to UTF-16 offsets.
/// Collect warnings from ftml to highlight them in editor.
fn get_ftml_token<'index>(
    wikitext: &'index str,
    index: &LineIndex,
) -> (Vec<ExtractedToken<'index>>, Vec<Diagnostic>) {
    // Utf16IndexMap has no entries for empty text, get_index would panic.
    if wikitext.is_empty() {
        return (vec![], vec![]);
    }

    let tokens = tokenize(wikitext);

    let (_, warnings) = parse(&tokens, &DEFAULT_PAGE_INFO, &DEFAULT_SETTINGS).into();

    let diagnostics: Vec<Diagnostic> = warnings
        .iter()
        .filter_map(|w| {
            Some(Diagnostic {
                msg: format!("{}: {}", w.kind().name(), w.rule()),
                ty: w.kind(),
                span: from_index_to_span(index, w.span())?,
            })
        })
        .collect();

    (Vec::from(tokens), diagnostics)
}

#[cfg(test)]
mod tests {
    use crate::ftml_ast::span::from_utf8_to_pos;

    use super::*;

    #[test]
    fn line_col_at_eof() {
        let src = "中文abc\n中文";
        let index = LineIndex::new(src);
        let pos = from_utf8_to_pos(&index, src.len()).unwrap();
        assert_eq!(pos, (1, 2))
    }
}
