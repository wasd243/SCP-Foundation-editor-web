use crate::ftml_ast::{Span, SubLangMark};

use ftml::parsing::Token as FtmlToken;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct TokenTree {
    /// The whole slice of the combined text
    pub slice: String,
    /// The whole span of the combined text.
    pub span: Span,
    /// The token tree type.
    pub ty: TokenTreeTy,
    /// The lsp mark for sub languages highlights.
    pub sub_lang_mark: Option<SubLangMark>,
    /// `true` if the token tree has a close tag, `false` as default.
    #[serde(skip)]
    pub has_close_tag: bool,
}

pub(super) type Alias = FtmlToken;
#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TokenTreeTy {
    /// A token from ftml that don't need to be combined.
    ParseToken(FtmlToken),
    /// A `[[include]]` block.
    Include,
    /// A block with alias like:
    /// `[[div id="some-id" class="some-class"]]`;
    /// `[[module css]] ... [[/module]]`;
    BlockWithAlias(Option<Vec<Alias>>),
    /// A block without alias like: `[[note]]`
    BlockWithoutAlias,
    /// A block close tag like: `[[/div]]`; `[[/module]]`, whether it's a block with alias or not,
    /// it always without alias. If the user closes the block with an alia, a parse error occurs.
    BlockClose,
}
