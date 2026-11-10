use ftml::parsing::Token as TokenTy;
use serde::Serialize;
use std::ops::Range;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    lsp_builtin::{
        BlockKind, Param,
        regex::{CSS, HTML, MODULE},
    },
    lsp_highlight::{AbsoluteSpan, ToAbsoluteSpan as _, ToAbsoluteSpans},
    utils::has_diagnostic,
};

use super::{BuiltIn, ParamKind, visit};

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Blocks<'lsp> {
    pub span: Span,
    /// This contains parameters like `div` `class` `=`.
    pub block_parameters: Vec<Param>,
    /// This contains block contents, includes everything.
    pub block_content: Vec<BuiltIn<'lsp>>,
    /// BlockKind, which determines CSS/HTML highlights.
    pub block_ty: BlockKind,
    /// Only use in sub languages BlockKind, otherwise stay as `None`.
    /// An Optional Range which only record line numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_sub_lang_span: Option<Range<u32>>,
    #[serde(skip)]
    used_index: usize,
    /// Oh shit I forgot the close tags.
    /// Close tags would only be a Identifier in the lsp.
    pub close_tag: Option<&'lsp Token>,
}

impl<'lsp> Blocks<'lsp> {
    #[must_use]
    pub(super) fn from_parse_token(
        parse_token: &'lsp [Token],
        diagnostic: &'lsp [Diagnostic],
    ) -> Option<Self> {
        let mut iter = parse_token.iter();

        while let Some(token) = iter.next() {
            if diagnostic.iter().any(|d| has_diagnostic(token, d)) {
                return None;
            }

            let start_token = token;

            // cache block token parameters temporarily.
            let mut block_token_parameters = Vec::new();
            let mut end_token: Option<&Token> = None;
            let mut block_ty = BlockKind::default();

            // match a block start: such as `[[div ]]` `[[footnote]]` `[[span ]]` `[[html ]]`
            loop {
                let t = iter.next()?;

                match t.ty {
                    TokenTy::Identifier if MODULE.is_match(&t.slice) => {
                        block_token_parameters.push(t);

                        let next = iter.next()?;

                        if next.ty == TokenTy::RightBlock {
                            break;
                        }

                        // push after the check, to prevent we push `]]` into the parameters
                        block_token_parameters.push(next);

                        if next.ty == TokenTy::Whitespace {
                            let next2 = iter.next()?;
                            block_token_parameters.push(next2);

                            if next2.ty == TokenTy::Identifier && CSS.is_match(&next2.slice) {
                                block_ty = BlockKind::ModuleCss;
                            } else {
                                break;
                            }
                        }

                        continue;
                    }
                    TokenTy::Identifier if HTML.is_match(&t.slice) => block_ty = BlockKind::Html,
                    _ => {}
                }

                if t.ty == TokenTy::RightBlock {
                    // won't assign value to end_token, end_token will be assigned at `[[/ ]]<-`
                    break;
                }

                block_token_parameters.push(t);
            }

            let param_kind = ParamKind::match_param_kind(&block_token_parameters);

            let block_parameters = Param::from_tokens(block_token_parameters, &param_kind);

            let mut block_contents = Vec::new();

            let need_raw = matches!(block_ty, BlockKind::ModuleCss | BlockKind::Html);

            let mut close_tag = None;
            loop {
                let rest = iter.as_slice();
                let Some(t) = iter.next() else {
                    break;
                };

                if t.ty == TokenTy::LeftBlock && !need_raw {
                    let (node, used) = visit(rest, diagnostic);
                    block_contents.push(node);
                    // `t` is already consumed, skip the remaining used - 1 tokens
                    for _ in 1..used {
                        iter.next();
                    }
                    continue;
                }

                if t.ty == TokenTy::LeftBlockEnd
                    && let [name, end, ..] = iter.as_slice()
                    && is_valid_block_end(name.ty)
                    && end.ty == TokenTy::RightBlock
                    && match block_ty {
                        BlockKind::ModuleCss => MODULE.is_match(&name.slice),
                        BlockKind::Html => HTML.is_match(&name.slice),
                        BlockKind::Block => true,
                    }
                {
                    close_tag = iter.next(); // name
                    end_token = iter.next(); // `]]`
                    break;
                }

                if !need_raw {
                    block_contents.push(BuiltIn::Token(t));
                }
            }

            if let Some(end_token) = end_token {
                let used_index = parse_token.len() - iter.as_slice().len();

                let mut block = Self {
                    span: Span {
                        start: start_token.span.start,
                        end: end_token.span.end,
                    },
                    block_parameters,
                    block_content: block_contents,
                    block_ty,
                    block_sub_lang_span: None,
                    used_index,
                    close_tag,
                };

                if matches!(block.block_ty, BlockKind::ModuleCss | BlockKind::Html) {
                    // we don't need to escape the `[` `]` token because there's nothing stored in
                    // block_contents when `block_ty` is html or css.

                    // get the `html`/`css` sub languages highlights' range.
                    let first = block.span.start.0 + 1;
                    let last = block.span.end.0;
                    block.block_sub_lang_span = (first < last).then_some(first..last);
                }

                return Some(block);
            }
        }

        None
    }

    pub(super) fn token_len(&self) -> usize {
        self.used_index
    }
}

/// Check close tags like `[[/div]]` `[[/<]]`.
fn is_valid_block_end(ty: TokenTy) -> bool {
    matches!(
        ty,
        TokenTy::Identifier | TokenTy::Quote | TokenTy::Other | TokenTy::Equals
    )
}

/// # NOTE:
/// implement for parameters only, `span` left for collapsing, `block_sub_lang_span` is left for
/// sublanguage highlights, `block_contents` could contains everything, so `block_parameters`
/// contains in `block_contents` is also implemented.
impl<'lsp> ToAbsoluteSpans for Blocks<'lsp> {
    fn to_abs_spans(&self) -> Vec<AbsoluteSpan> {
        let mut abs_spans: Vec<AbsoluteSpan> = self
            .block_parameters
            .iter()
            .filter_map(|bp| bp.to_abs_span())
            .collect();

        if let Some(close_tag) = self.close_tag.and_then(|token| token.to_abs_span()) {
            abs_spans.push(close_tag);
        }

        abs_spans
    }
}
