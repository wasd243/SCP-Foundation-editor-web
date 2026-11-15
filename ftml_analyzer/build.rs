use anyhow::{Context, Result};
use std::env;
use std::process::Command;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=deno/src");
    println!("cargo:rerun-if-changed=deno/deno.json");
    println!("cargo:rerun-if-changed=deno/deno.lock");

    let out_dir = env::var("OUT_DIR")?;

    let status = Command::new("deno")
        .args(["compile", "--output"])
        .arg(format!("{out_dir}/ftml-wrapper"))
        .arg("src/main.ts")
        .current_dir("deno")
        .status()
        .context("failed to run `deno`, is it in PATH?")?;

    if !status.success() {
        anyhow::bail!("deno compile failed");
    }
    Ok(())
}
