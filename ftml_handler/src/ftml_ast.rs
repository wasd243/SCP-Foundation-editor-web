mod token;

use ftml::parsing::ExtractedToken;
use ftml::parsing::ParseErrorKind;
use ftml::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub use token::Token;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

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

    /// Checks that any line of code is from combination
    pub fn from_combination(&self) -> bool {
        self.tokens.iter().any(|token| token.from_combination)
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

impl From<&str> for ParseToken {
    /// Parse new token from wikitext
    fn from(src: &str) -> Self {
        let wikitext = src.to_string();
        let (tokens, diagnostics) = get_ftml_token(&wikitext);
        let tokens = tokens
            .into_iter()
            .map(|token| Token::from_extracted_tokens(token))
            .collect();

        Self {
            tokens,
            diagnostics,
        }
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
fn get_ftml_token(wikitext: &str) -> (Vec<ExtractedToken<'_>>, Vec<Diagnostic>) {
    // Utf16IndexMap has no entries for empty text, get_index would panic.
    if wikitext.is_empty() {
        return (vec![], vec![]);
    }

    let map = ftml::Utf16IndexMap::new(wikitext);

    let tokens = tokenize(wikitext);

    let parse_result = parse(&tokens, &DEFAULT_PAGE_INFO, &DEFAULT_SETTINGS);

    let (_, warnings) = parse_result.into();

    let diagnostics: Vec<Diagnostic> = warnings
        .iter()
        .map(|warning| {
            let sp = warning.span();
            Diagnostic {
                msg: format!("{}: {}", warning.kind().name(), warning.rule()),
                ty: warning.kind(),
                span: Span {
                    start: map.get_index(sp.start),
                    end: map.get_index(sp.end),
                },
            }
        })
        .collect();

    let tokens: Vec<ExtractedToken<'_>> = tokens.into();
    let tokens: Vec<ExtractedToken<'_>> = tokens
        .iter()
        .map(|token| token.to_utf16_indices(&map))
        .collect::<Vec<ExtractedToken>>();

    (tokens, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FTML1: &str = r#"[[div id="some-id"]]
some context
[[/div]]"#;

    const FTML2: &str = r#"[[div class="div-without-close-tag]]
some context
[!--no close tag here, should warning--]"#;

    #[test]
    fn dbg_token1() {
        let src = FTML1.to_string();
        let tokens = get_ftml_token(&src);
        dbg!(tokens);
    }

    #[test]
    fn dbg_token2() {
        let src = FTML2.to_string();
        let tokens = get_ftml_token(&src);
        dbg!(tokens);
    }
}
