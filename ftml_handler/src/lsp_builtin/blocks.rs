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
    /// This contains parameters like `div` `class` `=`.
    block_paramters: Vec<&'lsp Token>,
    /// This contains block contents, includes everything.
    block_content: Vec<BuiltIn<'lsp>>,
    /// BlockKind, which determines CSS/HTML highlights.
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

            let need_raw = matches!(block_ty, BlockKind::ModuleCss | BlockKind::Html);

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
                        _ => true,
                    }
                {
                    iter.next(); // name
                    end_token = iter.next(); // `]]`
                    break;
                }

                block_contents.push(BuiltIn::Token(t));
            }

            if let Some(end_token) = end_token {
                let used_index = parse_token.len() - iter.as_slice().len();

                let mut block = Self {
                    span: Span {
                        start: start_token.span.start,
                        end: end_token.span.end,
                    },
                    block_paramters: block_parameters,
                    block_content: block_contents,
                    block_ty,
                    used_index,
                };

                if matches!(block.block_ty, BlockKind::ModuleCss | BlockKind::Html) {
                    Self::token_escape(&mut block);
                }

                return Some(block);
            }
        }

        None
    }

    pub(super) fn token_len(&self) -> usize {
        self.used_index
    }

    /// Escape `[` and `]`.
    fn token_escape(&mut self) {
        for builtin in &mut self.block_content {
            match builtin {
                BuiltIn::Token(t) => {
                    let slice = t.slice.replace('[', "\\[").replace(']', "\\]");
                    let span = Span {
                        start: t.span.start,
                        end: t.span.end,
                    };
                    *builtin = BuiltIn::EscapedToken { slice, span };
                }
                _ => unreachable!(
                    "token must be escaped in `html` and `css` blocks, found another BuiltIn"
                ),
            }
        }
    }
}

fn is_valid_block_end(ty: TokenTy) -> bool {
    matches!(
        ty,
        TokenTy::Identifier | TokenTy::Quote | TokenTy::Other | TokenTy::Equals
    )
}
