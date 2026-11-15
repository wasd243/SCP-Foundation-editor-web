use anyhow::{Result, anyhow};
use deno_core::{JsRuntime, ModuleSpecifier, RuntimeOptions, v8};
use lsp_types::{CompletionList, Diagnostic, Hover, Position};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use tokio::runtime::{Builder, EnterGuard, Runtime};

static BUNDLE: &str = include_str!(concat!(env!("OUT_DIR"), "/bundle.js"));

#[derive(Debug, Deserialize)]
pub struct EmbeddedToken {
    pub line: u32,
    pub character: u32,
    pub length: u32,
    pub kind: String,
}

pub struct WebService {
    rt: JsRuntime,
    _guard: EnterGuard<'static>,
    async_rt: &'static Runtime,
}

impl WebService {
    pub fn new() -> Result<Self> {
        let async_rt: &'static Runtime = Box::leak(Box::new(
            Builder::new_current_thread().enable_all().build()?,
        ));
        let guard = async_rt.enter();

        let mut rt = JsRuntime::new(RuntimeOptions::default());
        let specifier = ModuleSpecifier::parse("file:///bundle.js")?;
        let module_id = async_rt.block_on(rt.load_main_es_module_from_code(&specifier, BUNDLE))?;
        async_rt.block_on(rt.mod_evaluate(module_id))?;

        Ok(Self {
            rt,
            _guard: guard,
            async_rt,
        })
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

    pub fn tokens(&mut self, lang: &str, text: &str) -> Result<Vec<EmbeddedToken>> {
        self.call(lang, "tokens", text, Position::new(0, 0))
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
            .async_rt
            .block_on(async { self.rt.execute_script("call.js", format!("handle({req})")) })?;

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
