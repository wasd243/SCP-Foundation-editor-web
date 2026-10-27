use ftml::parsing::ExtractedToken;
use ftml::prelude::*;
use serde::Serialize;
use serde_json::Value;

use std::fmt::{Display, Formatter};
use std::sync::LazyLock;

#[derive(Serialize, Debug, Default)]
pub struct ParseToken {
    /// ftml tokens
    pub tokens: Value,
    /// ftml parse warnings
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize, Debug, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, Debug, Default)]
pub struct Diagnostic {
    pub msg: String,
    pub span: Span,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)] // left `Backward`
pub(crate) enum ValueIterDirection {
    Forward,
    Backward,
}

impl ParseToken {
    pub fn clean(&mut self) {
        if let Some(tokens) = self.tokens.as_array_mut() {
            tokens.retain(|token| {
                !matches!(
                    token.get("token").and_then(Value::as_str),
                    Some(
                        "input-start"
                            | "input-end"
                            | "line-break"
                            | "whitespace"
                            | "paragraph-break"
                    )
                )
            })
        }
    }

    /// Iterate over tokens before/after `index`, nearest first.
    pub(crate) fn value_iter(
        &self,
        index: usize,
        direction: ValueIterDirection,
    ) -> Box<dyn Iterator<Item = &Value> + '_> {
        let arr: &[Value] = self.tokens.as_array().map(Vec::as_slice).unwrap_or(&[]);
        let index = index.min(arr.len());

        if direction == ValueIterDirection::Forward {
            let start = (index + 1).min(arr.len());
            Box::new(arr[start..].iter())
        } else {
            Box::new(arr[..index].iter().rev())
        }
    }

    /// Iterate over tokens before/after `index` and then output with `index`.
    pub(crate) fn value_iter_indexed(
        &self,
        index: usize,
        direction: ValueIterDirection,
    ) -> Box<dyn Iterator<Item = (usize, &Value)> + '_> {
        let arr: &[Value] = self.tokens.as_array().map(Vec::as_slice).unwrap_or(&[]);
        let index = index.min(arr.len());

        if direction == ValueIterDirection::Forward {
            let start = (index + 1).min(arr.len());
            Box::new(arr.iter().enumerate().skip(start))
        } else {
            Box::new(arr[..index].iter().enumerate().rev())
        }
    }
}

#[macro_export]
macro_rules! value_iter_next {
    ($token:ident, $iter:ident) => {
        let Some((_, $token)) = $iter.next() else {
            return;
        };
    };
}

impl Display for ParseToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tokens)
    }
}

impl From<&str> for ParseToken {
    /// Parse new token from wikitext
    fn from(src: &str) -> Self {
        let wikitext = src.to_string();
        let (tokens, diagnostics) = get_ftml_token(&wikitext);

        Self {
            tokens: serde_json::to_value(tokens).expect("Should not fail"),
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
        let mut src = FTML1.to_string();
        let tokens = get_ftml_token(&mut src);
        dbg!(tokens);
    }

    #[test]
    fn dbg_token2() {
        let mut src = FTML2.to_string();
        let tokens = get_ftml_token(&mut src);
        dbg!(tokens);
    }
}
