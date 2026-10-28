mod token;
mod token_tree;

use token_tree::Alias;

use ftml::parsing::{ExtractedToken, Token as FtmlToken};
use ftml::prelude::*;
use serde::Serialize;
use std::sync::LazyLock;

use crate::ftml_block_identifier::regex::INCLUDE;

pub use token::{IncludeFoldMark, SubLangMark, Token};
pub use token_tree::{TokenTree, TokenTreeTy};

#[derive(Serialize, Debug)]
pub struct ParseTokenTree {
    /// ftml tokens but combined into trees with context.
    pub token_trees: Vec<TokenTree>,
    /// ftml parse warnings, keeps the same as `ParseToken`
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseTokenTree {
    pub fn from_parse_tokens(parse_tokens: ParseToken) -> Self {
        let tokens = parse_tokens.tokens;
        let mut trees = Vec::new();
        let mut index = 0;

        while index < tokens.len() {
            let token = &tokens[index];

            if token.ty != FtmlToken::LeftBlock && token.ty != FtmlToken::LeftBlockEnd {
                trees.push(TokenTree {
                    slice: token.slice.clone(),
                    span: token.span,
                    ty: TokenTreeTy::ParseToken(token.ty),
                    has_close_tag: false,
                });
                index += 1;
                continue;
            }

            // find the closing `]]` of this block tag
            let close_index = tokens[index..]
                .iter()
                .position(|candidate| candidate.ty == FtmlToken::RightBlock)
                .map(|offset| index + offset);

            let Some(close_index) = close_index else {
                // unclosed tag: keep the token as it was
                trees.push(TokenTree {
                    slice: token.slice.clone(),
                    span: token.span,
                    ty: TokenTreeTy::ParseToken(token.ty),
                    has_close_tag: false,
                });
                index += 1;
                continue;
            };

            let block_tokens = &tokens[index..=close_index];
            let slice: String = block_tokens
                .iter()
                .map(|inner| inner.slice.as_str())
                .collect();
            let span = Span {
                start: block_tokens[0].span.start,
                end: block_tokens[block_tokens.len() - 1].span.end,
            };

            let inner_tokens = &block_tokens[1..block_tokens.len() - 1];
            let name = inner_tokens
                .iter()
                .find(|inner| inner.ty == FtmlToken::Identifier);
            let is_include = name.is_some_and(|inner| INCLUDE.is_match(&inner.slice));

            let ty = if token.ty == FtmlToken::LeftBlockEnd {
                TokenTreeTy::BlockClose
            } else if is_include {
                TokenTreeTy::Include
            } else {
                let alias: Vec<Alias> = inner_tokens
                    .iter()
                    .skip_while(|inner| inner.ty != FtmlToken::Identifier)
                    .skip(1)
                    .filter(|inner| inner.ty != FtmlToken::Whitespace)
                    .map(|inner| inner.ty)
                    .collect();
                if alias.is_empty() {
                    TokenTreeTy::BlockWithoutAlias
                } else {
                    TokenTreeTy::BlockWithAlias(Some(alias))
                }
            };

            trees.push(TokenTree {
                slice,
                span,
                ty,
                has_close_tag: false,
            });
            index = close_index + 1;
        }

        Self {
            token_trees: trees,
            diagnostics: parse_tokens.diagnostics,
        }
    }
}

#[derive(Serialize, Debug, Default)]
pub struct ParseToken {
    /// ftml tokens
    pub tokens: Vec<Token>,
    /// ftml parse warnings
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize, Debug, Default, Copy, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, Debug, Default)]
pub struct Diagnostic {
    pub msg: String,
    pub span: Span,
}

impl ParseToken {
    pub fn clean(&mut self) {
        use ftml::parsing::Token;

        self.tokens.retain(|token| {
            !matches!(
                token.ty,
                Token::InputStart | Token::InputEnd
            )
        });
    }

    /// Iterate over tokens before/after `index` and then output with `index`.
    pub(crate) fn value_iter(&self, index: usize) -> impl Iterator<Item = (usize, &Token)> + '_ {
        self.tokens.iter().enumerate().skip(index + 1)
    }

    /// Checks that any line of code is from a combination
    pub fn from_combination(&self) -> bool {
        self.tokens.iter().any(|token| token.from_combination)
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
