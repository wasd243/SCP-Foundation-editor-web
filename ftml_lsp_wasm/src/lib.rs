use std::{cell::RefCell, fmt::Display};

use ftml_lsp::{Buffers, build_diags, match_requests, server_capabilities};
use lsp_server::{Message, Notification, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    PublishDiagnosticsParams,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
        PublishDiagnostics,
    },
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Default)]
struct WasmServer {
    buffers: Buffers,
}

impl WasmServer {
    fn handle_message(&mut self, message: Message) -> Result<Vec<Message>, JsValue> {
        match message {
            Message::Request(request) => {
                let response = match request.method.as_str() {
                    "initialize" => Response::new_ok(
                        request.id,
                        serde_json::json!({ "capabilities": server_capabilities() }),
                    ),
                    "shutdown" => Response::new_ok(request.id, ()),
                    _ => match_requests(request, &self.buffers),
                };
                Ok(vec![Message::Response(response)])
            }
            Message::Notification(notification) => self.handle_notification(notification),
            Message::Response(_) => Ok(Vec::new()),
        }
    }

    fn handle_notification(&mut self, notification: Notification) -> Result<Vec<Message>, JsValue> {
        let publish_uri = match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams = notification
                    .extract(DidOpenTextDocument::METHOD)
                    .map_err(js_error)?;
                let uri = params.text_document.uri;
                self.buffers.insert(uri.clone(), params.text_document.text);
                Some(uri)
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = notification
                    .extract(DidChangeTextDocument::METHOD)
                    .map_err(js_error)?;
                let uri = params.text_document.uri;
                if let Some(change) = params.content_changes.into_iter().last() {
                    self.buffers.insert(uri.clone(), change.text);
                }
                Some(uri)
            }
            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams = notification
                    .extract(DidCloseTextDocument::METHOD)
                    .map_err(js_error)?;
                self.buffers.remove(&params.text_document.uri);
                Some(params.text_document.uri)
            }
            _ => None,
        };

        let Some(uri) = publish_uri else {
            return Ok(Vec::new());
        };
        let diagnostics = self
            .buffers
            .get(&uri)
            .map(|source| build_diags(ftml_analyzer::get_ast(source)))
            .unwrap_or_default();
        let params = PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        };
        Ok(vec![Message::Notification(Notification::new(
            PublishDiagnostics::METHOD.to_string(),
            params,
        ))])
    }
}

#[derive(Serialize)]
struct JsonRpcMessage<'a> {
    jsonrpc: &'static str,
    #[serde(flatten)]
    message: &'a Message,
}

thread_local! {
    static SERVER: RefCell<WasmServer> = RefCell::new(WasmServer::default());
}

#[wasm_bindgen]
pub fn reset() {
    SERVER.with_borrow_mut(|server| *server = WasmServer::default());
}

#[wasm_bindgen]
pub fn handle_message(input: &str) -> Result<String, JsValue> {
    let message: Message = serde_json::from_str(input).map_err(js_error)?;
    SERVER.with_borrow_mut(|server| {
        let replies = server.handle_message(message)?;
        let json_rpc: Vec<_> = replies
            .iter()
            .map(|message| JsonRpcMessage {
                jsonrpc: "2.0",
                message,
            })
            .collect();
        serde_json::to_string(&json_rpc).map_err(js_error)
    })
}

fn js_error(error: impl Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(test)]
mod tests {
    use lsp_server::{Message, Request, RequestId};
    use serde_json::json;

    use super::WasmServer;

    #[test]
    fn initialize_returns_lsp_capabilities() {
        let mut server = WasmServer::default();
        let replies = server
            .handle_message(Message::Request(Request::new(
                RequestId::from(1),
                "initialize".to_owned(),
                json!({}),
            )))
            .expect("initialize should succeed");

        assert_eq!(replies.len(), 1);

        let Message::Response(response) = &replies[0] else {
            panic!("expected an initialize response");
        };

        assert!(response.response_result.is_ok());
    }

    #[test]
    fn did_close_clears_document_diagnostics() {
        let mut server = WasmServer::default();
        let open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ftml",
                    "languageId": "ftml",
                    "version": 1,
                    "text": "[[div]]text[[/div]]"
                }
            }
        });
        let message = serde_json::from_value(open).expect("didOpen should deserialize");
        assert_eq!(server.handle_message(message).unwrap().len(), 1);

        let close = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": { "textDocument": { "uri": "file:///test.ftml" } }
        });
        let message = serde_json::from_value(close).expect("didClose should deserialize");
        let replies = server.handle_message(message).unwrap();
        assert_eq!(replies.len(), 1);
        let Message::Notification(notification) = &replies[0] else {
            panic!("expected a diagnostics notification");
        };
        assert_eq!(
            notification.params["diagnostics"],
            serde_json::Value::Array(Vec::new())
        );
    }
}
