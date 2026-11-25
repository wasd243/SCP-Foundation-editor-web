#![warn(clippy::all)]
#![expect(clippy::mutable_key_type)]

mod activities;

use activities::main_loop;

use anyhow::Result;
use lsp_server::Connection;
use lsp_types::{
    CompletionOptions, FoldingRangeProviderCapability, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use std::collections::HashMap;

const COLLAPSED_TEXT: &str = "...";
type Buffers = HashMap<Uri, String>;

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        #[cfg(feature = "highlight")]
        semantic_tokens_provider: Some(ftml_analyzer::semantic_tokens_capability()),
        completion_provider: Some(CompletionOptions::default()),
        ..ServerCapabilities::default()
    }
}

/// Build the LSP server.
pub fn build_server() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    connection.initialize(serde_json::to_value(server_capabilities())?)?;

    main_loop(connection)?;
    io_threads.join()?;
    Ok(())
}
