use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    lsp_builtin::{
        BlockKind,
        regex::{CSS, HTML, MODULE},
    },
    lsp_highlight::{AbsoluteSpan, ToAbsoluteSpan as _, ToAbsoluteSpans},
    utils::{has_diagnostic, highlight::color_nodes},
};

use super::{BuiltIn, Param, ParamKind, visit};

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Blocks<'lsp> {
    pub span: Span,
    /// This contains parameters like `div` `class` `=`.
    pub block_parameters: Vec<Param>,
    /// This contains block contents, includes everything.
    pub block_contents: Vec<BuiltIn<'lsp>>,
    /// BlockKind, which determines CSS/HTML highlights.
    pub block_ty: BlockKind,
    /// Only use in sub languages BlockKind, otherwise stay as `None`.
    /// An Optional Range which only record line numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_sub_lang_span: Option<Span>,
    #[serde(skip)]
    used_index: usize,
    /// Oh shit I forgot the close tags.
    /// Close tags would only be a Identifier in the lsp.
    pub close_tag: Option<&'lsp Token>,
    /// [[ ]] [[/ ]]
    pub delimiters: [&'lsp Token; 4],
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

            // cache block token parameters temporarily.
            // we don't need to rename start token, the `token` variable is the start of the block token.
            let mut block_token_parameters = Vec::new();
            let mut end_token: Option<&Token> = None;
            let mut block_ty = BlockKind::default();

            let mut end: Option<&Token> = None;
            let mut block_end_start_token: Option<&Token> = None;

            // match a block start: such as `[[div ]]` `[[footnote]]` `[[span ]]` `[[html ]]`
            loop {
                let t = iter.next()?;

                match t.ty {
                    TokenTy::Identifier if MODULE.is_match(&t.slice) => {
                        block_token_parameters.push(t);

                        let next = iter.next()?;

                        if next.ty == TokenTy::RightBlock {
                            // push the `]]` into container
                            end = Some(next);
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
                    // this is the same token as previous, should NEVER repeat because previous code
                    // already break the loop.
                    end = Some(t);
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
                    block_end_start_token = Some(t); // [[/
                    close_tag = iter.next(); // name
                    end_token = iter.next(); // `]]`
                    break;
                }

                if !need_raw {
                    block_contents.push(BuiltIn::Token(t));
                }
            }

            let end2 = Some(end_token?);
            let starts_and_ends_of_the_block = [token, end?, block_end_start_token?, end2?];

            if let Some(end_token) = end_token {
                let used_index = parse_token.len() - iter.as_slice().len();

                let mut blocks = Self {
                    span: Span {
                        start: token.span.start,
                        end: end_token.span.end,
                    },
                    block_parameters,
                    block_contents,
                    block_ty,
                    block_sub_lang_span: None,
                    used_index,
                    close_tag,
                    delimiters: starts_and_ends_of_the_block,
                };

                calc_sub_highlight_span(&mut blocks);

                return Some(blocks);
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

        // We need to handle Identifiers specifically in `close_tag`.
        if let Some(close_tag) = self
            .close_tag
            .and_then(|token| token.with_idents_and_symbols_to_abs_span())
        {
            abs_spans.push(close_tag);
        }

        abs_spans.extend(self.delimiters.iter().filter_map(|t| t.to_abs_span()));

        for content in &self.block_contents {
            color_nodes(content, &mut abs_spans);
        }

        abs_spans
    }
}

/// Calculate sublanguage highlights' spans.
fn calc_sub_highlight_span(blocks: &mut Blocks) {
    if matches!(blocks.block_ty, BlockKind::ModuleCss | BlockKind::Html) {
        // we don't need to escape the `[` `]` token because there's nothing stored in
        // block_contents when `block_ty` is html or css.

        let first = (blocks.span.start.0 + 1, 0_u32);
        let last = (blocks.span.end.0 - 1, blocks.span.end.1);
        blocks.block_sub_lang_span = Some(Span {
            start: first,
            end: last,
        });
    }
}
