use ftml::parsing::ExtractedToken;
use serde::Serialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Debug)]
pub struct ParseToken {
    /// ftml tokens
    pub tokens: Value,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)] // left `Backward` for enum
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
                    Some("input-start" | "input-end" | "line-break" | "whitespace")
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
        let tokens = get_ftml_token(&wikitext);

        Self {
            tokens: serde_json::to_value(tokens).expect("Should not fail"),
        }
    }
}

/// Tokenize without preprocessing, so spans match the editor text.
/// Spans are converted from UTF-8 byte offsets to UTF-16 offsets.
fn get_ftml_token(wikitext: &str) -> Vec<ExtractedToken<'_>> {
    // Utf16IndexMap has no entries for empty text, get_index would panic.
    if wikitext.is_empty() {
        return Vec::new();
    }

    let tokens: Vec<ExtractedToken<'_>> = ftml::tokenize(wikitext).into();
    let map = ftml::Utf16IndexMap::new(wikitext);

    tokens
        .iter()
        .map(|token| token.to_utf16_indices(&map))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FTML: &str = r#"[[div id="some-id"]]
some context
[[/div]]"#;

    const EXPECT: &str = include_str!("../tests/tokens.json");

    #[test]
    fn check_token() {
        let mut src = FTML.to_string();
        let tokens = get_ftml_token(&mut src);
        let tokens = format!("{:?}", tokens);
        assert_eq!(tokens, EXPECT);
    }
}
