//! A file in `tests/` should be written in:
//! ```fttml, ignore
//! tests/
//!     |- your tests(use block name)
//!         |- input.ftml (write it by yourself)
//!         |- expect.toml (write it by yourself)
//!         |- builtin.json (auto generated)
//! ```
use anyhow::{Context, Result};
use ftml_analyzer::get_ast;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;

use super::Expect;

const INPUT_FILE: &str = "input.ftml";
const EXPECT_FILE: &str = "expect.toml";
const BUILTIN_FILE: &str = "builtin.json";

#[derive(Debug)]
pub struct TestUnit {
    dir: PathBuf,
    pub name: String,
    pub input: String,
    pub expect: Option<String>,
    pub builtin: Option<Value>,
}

impl TestUnit {
    /// Read from one test unit.
    pub fn read_from_single_dir(dir: PathBuf) -> Result<Self> {
        let name = dir
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .with_context(|| "should have a folder name")?;

        let input = fs::read_to_string(dir.join(INPUT_FILE))
            .with_context(|| format!("needs an `{INPUT_FILE}` in {}", dir.display()))?;

        let expect = read_optional(&dir.join(EXPECT_FILE))?;

        let builtin = read_optional(&dir.join(BUILTIN_FILE))?
            .map(|s| serde_json::from_str::<Value>(&s))
            .transpose()?;

        Ok(Self {
            dir,
            name,
            input,
            expect,
            builtin,
        })
    }

    /// Bless builtin JSON based on input ftml source code.
    pub fn bless(&mut self) -> Result<()> {
        let value = serde_json::to_value(get_ast(&self.input))?;
        fs::write(
            self.dir.join(BUILTIN_FILE),
            serde_json::to_string_pretty(&value)?,
        )?;
        self.builtin = Some(value);
        Ok(())
    }

    /// Check an item.
    pub fn check(&self) -> Result<bool> {
        let ast = get_ast(&self.input);

        // snapshot, required
        let actual = serde_json::to_value(&ast)?;
        let Some(snapshot) = &self.builtin else {
            anyhow::bail!("no `{BUILTIN_FILE}` snapshot, bless before check");
        };
        let snapshot_ok = actual == *snapshot;

        // expectations, optional
        let expect_ok = match &self.expect {
            Some(text) => Expect::from_str(text)?
                .expectation
                .iter()
                .all(|e| ast.nodes.iter().any(|n| e.matches_node(n))),
            None => true,
        };

        Ok(snapshot_ok && expect_ok)
    }
}

/// Read a file if it exists. Missing file is `None`, other errors propagate.
fn read_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
