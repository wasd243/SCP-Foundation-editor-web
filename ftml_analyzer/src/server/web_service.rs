use anyhow::{Result, anyhow};
use deno_core::{JsRuntime, RuntimeOptions, v8};
use lsp_types::Position;
use serde_json::{Value, json};

static BUNDLE: &str = include_str!(concat!(env!("OUT_DIR"), "/bundle.js"));

/// This struct is set for running HTML/CSS LSPs.
pub struct WebService {
    rt: JsRuntime,
}

impl WebService {
    pub fn new() -> Result<Self> {
        let mut rt = JsRuntime::new(RuntimeOptions::default());
        rt.execute_script("bundle.js", BUNDLE)?;
        Ok(Self { rt })
    }

    pub fn call(
        &mut self,
        lang: &str,
        method: &str,
        text: &str,
        position: Position,
    ) -> Result<Value> {
        let req = json!({
            "id": 0, "lang": lang, "method": method,
            "text": text, "position": position,
        })
        .to_string();

        // Serialize again
        let code = format!("handle({})", serde_json::to_string(&req)?);
        let global = self.rt.execute_script("call.js", code)?;

        let out = {
            deno_core::scope!(scope, &mut self.rt);
            let local = v8::Local::new(scope, global);
            local.to_rust_string_lossy(scope)
        };

        let mut resp: Value = serde_json::from_str(&out)?;
        if let Some(e) = resp.get("error") {
            return Err(anyhow!("{e}"));
        }
        Ok(resp["result"].take())
    }
}
