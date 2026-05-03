pub mod ftml_ast;

use crate::ftml_ast::ParseToken;

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn tokenization(src: &str) -> Result<JsValue, JsValue> {
    let mut tokens = ParseToken::from(src);
    tokens.clean();

    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    tokens
        .tokens
        .serialize(&serializer)
        .map_err(|err| JsValue::from_str(&err.to_string()))
}
