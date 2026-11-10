#![warn(clippy::all)]
#![warn(unused_lifetimes)]

pub mod ftml_ast;
pub mod lsp_builtin;
pub mod lsp_highlight;
pub mod server;
pub mod utils;

use crate::{
    ftml_ast::ParseToken,
    lsp_builtin::LspBuiltIn,
    lsp_highlight::AbsoluteSpanTree,
    server::{SERVER_CAPABILITIES, main_loop},
};

use anyhow::{Ok, Result};
use lsp_server::Connection;
use lsp_types::SemanticToken;
use serde::Serialize;
use wasm_bindgen::prelude::*;

pub use crate::ftml_ast::Span;

/// A macro to avoid repeating code.
macro_rules! set_up_parsing {
    ($src:ident, $out:ident) => {
        let mut tokens = ParseToken::from($src);
        tokens.clean();
        tokens.mute_ftml_fps();

        let mut $out = LspBuiltIn::visitor(&tokens);
        $out.sanitize_whitespace();
    };
}

#[wasm_bindgen]
/// Same function for WASM.
pub fn tokenization(src: &str) -> Result<JsValue, JsValue> {
    set_up_parsing!(src, lsp_built_in);

    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    lsp_built_in
        .serialize(&serializer)
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

/// A function for parsing token to builin tree JSON.
///
/// NOTE: this function is set for debug only.
pub fn debug_parse(src: &str) -> Result<String, serde_json::Error> {
    set_up_parsing!(src, lsp_built_in);

    let highlight_abs_token = AbsoluteSpanTree::from_builtin_tree(lsp_built_in.tree);

    serde_json::to_string_pretty(&highlight_abs_token)
}

/// Main LSP parsing function.
/// From the source code to IDE acceptable Semantic Tokens.
pub fn parse(src: &str) -> Vec<SemanticToken> {
    set_up_parsing!(src, lsp_built_in);
    AbsoluteSpanTree::from_builtin_tree(lsp_built_in.tree).into_semantic_tokens()
}

/// A function to get `ParseToken`.
pub fn parse_token(src: &str) -> ParseToken {
    let mut tokens = ParseToken::from(src);
    tokens.clean();
    tokens.mute_ftml_fps();
    tokens
}

/// Build the LSP server.
pub fn build_server() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    connection.initialize(serde_json::to_value(&*SERVER_CAPABILITIES)?)?;

    main_loop(connection)?;
    io_threads.join()?;
    Ok(())
}
