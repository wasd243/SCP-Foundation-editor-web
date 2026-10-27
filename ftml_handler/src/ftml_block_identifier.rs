mod blocks;
mod regex;
mod utils;

use crate::ftml_ast::ParseToken;
use ftml::parsing::Token;

pub(crate) const IDENTIFIER: Token = Token::Identifier;
pub(crate) const LEFT_BLOCK: Token = Token::LeftBlock;
pub(crate) const RIGHT_BLOCK: Token = Token::RightBlock;
pub(crate) const LEFT_BLOCK_END: Token = Token::LeftBlockEnd;

impl ParseToken {
    pub fn mark_blocks(&mut self) {
        let len = self.tokens.len();
        for i in 0..len {
            ////////////////Include blocks collapse///////////////////
            self.mark_includes(i);
            //////////////////////////////////////////////////////////
            
            ////////////////Sub Language Highlight////////////////////
            self.mark_module_css(i);
            self.mark_html(i);
            //////////////////////////////////////////////////////////
        }
    }
}
