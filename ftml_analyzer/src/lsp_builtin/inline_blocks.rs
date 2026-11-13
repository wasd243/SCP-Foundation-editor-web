use ftml::parsing::Token as TokenTy;
use serde::Serialize;

use crate::{
    Span,
    ftml_ast::{Diagnostic, Token},
    lsp_highlight::{AbsoluteSpan, ToAbsoluteSpan, ToAbsoluteSpans},
    utils::check_diagnostic_without_include,
};

use super::{ParamKind, types::Param};

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct InlineBlocks<'lsp> {
    pub span: Span,
    /// [[
    pub start_block: &'lsp Token,
    /// ]]
    pub end_block: &'lsp Token,
    /// Keep content for sub hihgliht in `[[include]]` `[[user name]]`.
    pub parameters: Vec<Param>,
}

impl<'lsp> InlineBlocks<'lsp> {
    #[must_use]
    pub(super) fn from_parse_token_slice(
        tokens: &'lsp [Token],
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
            start_block: start_mark,
            end_block: &rest[end_idx],
            parameters,
        })
    }

    pub(super) fn content_len(&self) -> usize {
        self.parameters.len()
    }
}

/// # NOTE:
/// This implementation ONLY convert parameters to `AbsoluteSpan`,
/// span, is left for collapsing.
impl ToAbsoluteSpans for InlineBlocks<'_> {
    fn to_abs_spans(&self) -> Vec<AbsoluteSpan> {
        let mut spans = Vec::with_capacity(self.parameters.len() + 2);
        spans.extend(self.start_block.to_abs_span());
        spans.extend(self.parameters.iter().filter_map(|p| p.to_abs_span()));
        spans.extend(self.end_block.to_abs_span());
        spans
    }
}
