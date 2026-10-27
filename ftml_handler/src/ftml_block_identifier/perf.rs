use crate::ftml_ast::{ParseToken, Token};

impl ParseToken {
    /// merge consecutive tokens that carry the same sub-language mark into one token
    pub(super) fn combine_sub_languages_tokens(&mut self) {
        let old_tokens: Vec<Token> = std::mem::take(&mut self.tokens);
        let mut combined_tokens: Vec<Token> = Vec::with_capacity(old_tokens.len());

        for token in old_tokens {
            if let Some(previous_token) = combined_tokens.last_mut()
                && previous_token.sub_lang_mark.is_some()
                && previous_token.sub_lang_mark == token.sub_lang_mark
            {
                previous_token.slice.push_str(&token.slice);
                previous_token.span.end = token.span.end;
                previous_token.from_combination = true;
                continue;
            }

            combined_tokens.push(token);
        }

        self.tokens = combined_tokens;
    }
}
