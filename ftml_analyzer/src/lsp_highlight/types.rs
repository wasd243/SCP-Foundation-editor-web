use lsp_types::{
    SemanticTokenModifier, SemanticTokenType, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensServerCapabilities,
};

/// Register every predefined type; the index is the position in this slice.
pub const TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::TYPE,
    SemanticTokenType::CLASS,
    SemanticTokenType::ENUM,
    SemanticTokenType::INTERFACE,
    SemanticTokenType::STRUCT,
    SemanticTokenType::TYPE_PARAMETER,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::ENUM_MEMBER,
    SemanticTokenType::EVENT,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::METHOD,
    SemanticTokenType::MACRO,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::MODIFIER,
    SemanticTokenType::COMMENT,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::REGEXP,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::DECORATOR,
    // NOTE: ALL previous types are the default types in IDE, do NOT touch.
    SemanticTokenType::new("ftmlBold"),
    SemanticTokenType::new("ftmlItalic"),
    SemanticTokenType::new("ftmlUnderline"),
    SemanticTokenType::new("ftmlStrike"),
    SemanticTokenType::new("ftmlMonospace"),
    SemanticTokenType::new("ftmlSuperscript"),
    SemanticTokenType::new("ftmlSubscript"),
];

pub const MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::READONLY,
    SemanticTokenModifier::STATIC,
    SemanticTokenModifier::DEPRECATED,
    SemanticTokenModifier::ABSTRACT,
    SemanticTokenModifier::ASYNC,
    SemanticTokenModifier::MODIFICATION,
    SemanticTokenModifier::DOCUMENTATION,
    SemanticTokenModifier::DEFAULT_LIBRARY,
];

/// Initialize highlighter
pub(crate) fn semantic_tokens_capability() -> SemanticTokensServerCapabilities {
    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
        work_done_progress_options: Default::default(),
        legend: SemanticTokensLegend {
            token_types: TYPES.to_vec(),
            token_modifiers: MODIFIERS.to_vec(),
        },
        range: Some(false),
        full: Some(SemanticTokensFullOptions::Bool(true)),
    })
}
