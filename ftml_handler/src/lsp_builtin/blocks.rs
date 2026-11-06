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

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct Blocks<'lsp> {
    span: Span,
    block_paramters: Vec<&'lsp Token>,
    block_content: Vec<&'lsp Token>,
    blocks_inner: Vec<Blocks<'lsp>>,
    block_ty: BlockKind,
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
            let mut inners = Vec::new();
            let mut end_token: Option<&Token> = None;
            let mut block_ty = BlockKind::default();

            // match a block start: such as `[[div ]]` `[[footnote]]` `[[span ]]` `[[html ]]`
            loop {
                let Some(t) = iter.next() else { break };

                match t.ty {
                    TokenTy::Identifier if MODULE.is_match(&t.slice) => {
                        let Some(t) = iter.next() else {
                            break;
                        };
                        if t.ty == TokenTy::Identifier && CSS.is_match(&t.slice) {
                            block_ty = BlockKind::ModuleCss
                        }
                    }
                    TokenTy::Identifier if HTML.is_match(&t.slice) => block_ty = BlockKind::Html,
                    _ => block_ty = BlockKind::Block,
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

                if t.ty == TokenTy::LeftBlock
                    && let Some(nested) = Self::from_parse_token(rest, diagnostic)
                {
                    let n = nested.block_content.len();
                    block_contents.extend(&rest[..n + 2]); // start + content + end of nested
                    iter.nth(n); // skip nested content + end mark; t is already consumed
                    inners.push(nested);
                    continue;
                }

                if t.ty == TokenTy::LeftBlockEnd {
                    let Some(t) = iter.next() else {
                        break;
                    };

                    if t.ty == TokenTy::RightBlock {
                        end_token = Some(t);
                        break;
                    }
                }

                block_contents.push(t);
            }

            if let Some(end_token) = end_token {
                return Some(Self {
                    span: Span {
                        start: start_token.span.start,
                        end: end_token.span.end,
                    },
                    block_paramters: block_parameters,
                    block_content: block_contents,
                    blocks_inner: inners,
                    block_ty,
                });
            }
        }

        None
    }
}
