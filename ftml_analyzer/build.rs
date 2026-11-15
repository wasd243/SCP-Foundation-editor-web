use anyhow::{Context, Result};
use std::{env, path::PathBuf, process::Command};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=deno/src/main.ts");
    println!("cargo:rerun-if-changed=deno/src/deno.json");

    let out = PathBuf::from(env::var("OUT_DIR")?).join("bundle.js");
    let status = Command::new("deno")
        .args([
            "bundle",
            "--config",
            "deno/deno.json",
            "deno/src/main.ts",
            "-o",
        ])
        .arg(&out)
        .status()
        .context("Need to install deno CLI")?;

    if !status.success() {
        anyhow::bail!("Failed to build bundle");
    }

    Ok(())
}
