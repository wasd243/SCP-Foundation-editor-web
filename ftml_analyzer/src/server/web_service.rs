use anyhow::{Result, anyhow};
use deno_core::{JsRuntime, RuntimeOptions, v8};
use lsp_types::{CompletionList, Diagnostic, Hover, Position};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

static BUNDLE: &str = include_str!(concat!(env!("OUT_DIR"), "/bundle.js"));

pub struct WebService {
    rt: JsRuntime,
}

impl WebService {
    pub fn new() -> Result<Self> {
        let mut rt = JsRuntime::new(RuntimeOptions::default());
        rt.execute_script("bundle.js", BUNDLE)?;
        Ok(Self { rt })
    }

    pub fn diagnose(&mut self, lang: &str, text: &str) -> Result<Vec<Diagnostic>> {
        self.call(lang, "diagnose", text, Position::new(0, 0))
    }

    pub fn complete(&mut self, lang: &str, text: &str, pos: Position) -> Result<CompletionList> {
        self.call(lang, "complete", text, pos)
    }

    pub fn hover(&mut self, lang: &str, text: &str, pos: Position) -> Result<Option<Hover>> {
        self.call(lang, "hover", text, pos)
    }

    fn call<T: DeserializeOwned>(
        &mut self,
        lang: &str,
        method: &str,
        text: &str,
        position: Position,
    ) -> Result<T> {
        let req = json!({
            "lang": lang, "method": method,
            "text": text, "position": position,
        });
        let global = self
            .rt
            .execute_script("call.js", format!("handle({req})"))?;

        let out = {
            deno_core::scope!(scope, &mut self.rt);
            let local = v8::Local::new(scope, global);
            local.to_rust_string_lossy(scope)
        };

        let mut resp: Value = serde_json::from_str(&out)?;
        if let Some(e) = resp.get("error") {
            return Err(anyhow!("{e}"));
        }
        Ok(serde_json::from_value(resp["result"].take())?)
    }
}
