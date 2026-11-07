use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    lsp_builtin::{
        BlockKind,
        regex::{CSS, HTML, MODULE},
    },
    utils::has_diagnostic,
};

use super::{BuiltIn, visit};

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct Blocks<'lsp> {
    span: Span,
    block_paramters: Vec<&'lsp Token>,
    block_content: Vec<BuiltIn<'lsp>>,
    block_ty: BlockKind,
    #[serde(skip)]
    used_index: usize,
}

impl<'lsp> Blocks<'lsp> {
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
            let mut block_parameters = Vec::new();
            let mut end_token: Option<&Token> = None;
            let mut block_ty = BlockKind::default();

            // match a block start: such as `[[div ]]` `[[footnote]]` `[[span ]]` `[[html ]]`
            loop {
                let t = iter.next()?;

                match t.ty {
                    TokenTy::Identifier if MODULE.is_match(&t.slice) => {
                        block_parameters.push(t);

                        let next = iter.next()?;
                        block_parameters.push(next);

                        if next.ty == TokenTy::RightBlock {
                            break;
                        }

                        if next.ty == TokenTy::Whitespace {
                            let next2 = iter.next()?;
                            block_parameters.push(next2);

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

                block_parameters.push(t);
            }

            let mut block_contents = Vec::new();

            loop {
                let rest = iter.as_slice();
                let Some(t) = iter.next() else {
                    break;
                };

                if t.ty == TokenTy::LeftBlock {
                    let (node, used) = visit(rest, diagnostic);
                    block_contents.push(node);
                    // `t` is already consumed, skip the remaining used - 1 tokens
                    for _ in 1..used {
                        iter.next();
                    }
                    continue;
                }

                if t.ty == TokenTy::LeftBlockEnd {
                    let t = iter.next()?;

                    if t.ty == TokenTy::Identifier
                        && let Some(t) = iter.next()
                    {
                        end_token = Some(t);
                        break;
                    }
                }

                block_contents.push(BuiltIn::Token(t));
            }

            if let Some(end_token) = end_token {
                let used_index = parse_token.len() - iter.as_slice().len();

                return Some(Self {
                    span: Span {
                        start: start_token.span.start,
                        end: end_token.span.end,
                    },
                    block_paramters: block_parameters,
                    block_content: block_contents,
                    block_ty,
                    used_index,
                });
            }
        }

        None
    }

    pub(super) fn token_len(&self) -> usize {
        self.used_index
    }

    pub(super) fn combine_sub_lang_token(&mut self) {
        if self.block_ty == BlockKind::ModuleCss || self.block_ty == BlockKind::Html {
            let sub_lang_tokens = &self.block_content;

            for node in sub_lang_tokens {
                if !matches!(node, BuiltIn::Token(_)) {
                    panic!("Should never see other BuiltIn in `html` or `css` sub lang blocks.");
                }
            }
        }
    }
}
