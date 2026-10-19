pub mod ftml_ast;
pub mod ftml_block_identifier;

use crate::ftml_ast::ParseToken;

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn tokenization(src: &str) -> Result<JsValue, JsValue> {
    let mut tokens = ParseToken::from(src);
    tokens.clean();
    tokens.mark_sublanguage_blocks();

    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    tokens
        .tokens
        .serialize(&serializer)
        .map_err(|err| JsValue::from_str(&err.to_string()))
}
