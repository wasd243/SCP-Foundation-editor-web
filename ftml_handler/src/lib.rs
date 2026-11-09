#![warn(clippy::all)]

pub mod ftml_ast;
pub mod lsp;
pub mod lsp_builtin;
mod utils;

use crate::{ftml_ast::ParseToken, lsp_builtin::LspBuiltIn};

use serde::Serialize;
use wasm_bindgen::prelude::*;

pub use crate::ftml_ast::Span;

#[wasm_bindgen]
/// Same function for WASM.
pub fn tokenization(src: &str) -> Result<JsValue, JsValue> {
    let mut tokens = ParseToken::from(src);
    tokens.clean();
    tokens.mute_ftml_fps();

    let mut lsp_built_in = LspBuiltIn::visitor(&tokens);
    lsp_built_in.sanitize_whitespace();

    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    lsp_built_in
        .serialize(&serializer)
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

/// A function for parsing token to builin tree JSON.
pub fn parse(src: &str) -> Result<String, serde_json::Error> {
    let mut tokens = ParseToken::from(src);
    tokens.clean();
    tokens.mute_ftml_fps();

    let mut lsp_builtin = LspBuiltIn::visitor(&tokens);
    lsp_builtin.sanitize_whitespace();

    serde_json::to_string_pretty(&lsp_builtin)
}

/// A function to get `ParseToken`.
pub fn parse_token(src: &str) -> ParseToken {
    let mut tokens = ParseToken::from(src);
    tokens.clean();
    tokens.mute_ftml_fps();
    tokens
}
