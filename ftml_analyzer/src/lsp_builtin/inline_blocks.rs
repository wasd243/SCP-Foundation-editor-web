use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    lsp_builtin::Param,
    utils::check_diagnostic_without_include,
};

use super::ParamKind;

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct InlineBlocks {
    span: Span,
    /// Keep content for sub hihgliht in `[[include]]` `[[user name]]`.
    parameters: Vec<Param>,
}

impl InlineBlocks {
    #[must_use]
    pub(super) fn from_parse_token_slice(
        tokens: &[Token],
        diagnostics: &[Diagnostic],
    ) -> Option<Self> {
        let (start_mark, rest) = tokens.split_first()?;
        let end_idx = rest.iter().position(|t| t.ty == TokenTy::RightBlock)?;
        let end_span = rest[end_idx].span.end;

        for token in tokens[..=end_idx + 1].iter() {
            if diagnostics
                .iter()
                .any(|d| check_diagnostic_without_include(token, d))
            {
                return None;
            }
        }

        let token_parameters: Vec<&Token> = rest[..end_idx].iter().collect();
        let param_kind = ParamKind::match_param_kind(&token_parameters);
        let parameters = Param::from_tokens(token_parameters, &param_kind);

        Some(Self {
            span: Span {
                start: start_mark.span.start,
                end: end_span,
            },
            parameters,
        })
    }

    pub(super) fn content_len(&self) -> usize {
        self.parameters.len()
    }
}
