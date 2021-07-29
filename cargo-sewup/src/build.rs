use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result};
use hex::encode;
use tokio::{
    fs::{read, read_to_string, write},
    process::Command,
};

use crate::config::Toml;
use crate::{deploy_file, deploy_wasm};

async fn check_dependency() -> Result<()> {
    Command::new("wat2wasm")
        .stderr(Stdio::null())
        .output()
        .await
        .context("wat2wasm not found")?;
    Ok(())
}

async fn check_cargo_toml() -> Result<String> {
    let config_contents = read_to_string("Cargo.toml")
        .await
        .context("can not read Cargo.toml")?;
    let config: Toml = toml::from_str(config_contents.as_str())?;

    // TODO: more toml config checking here

    Ok(config.package.name.replace("-", "_"))
}

async fn build_runtime_wasm() -> Result<()> {
    Command::new("cargo")
        .args(&["build", "--release", "--target=wasm32-unknown-unknown"])
        .output()
        .await
        .context("fail to build runtime wasm")?;
    Ok(())
}

async fn generate_runtime_info(rt_wasm_path: String) -> Result<(usize, usize, String)> {
    let bin = read(Path::new(&rt_wasm_path))
        .await
        .context("fail to read runtime wasm")?;
    let bin_size = bin.len();
    let mem_size = bin_size / (64 * 1024) + 1;
    let hex_string = encode(bin)
        .chars()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|x| format!(r#"\{}{}"#, x[0], x[1]))
        .fold(String::new(), |mut acc, x| {
            acc.push_str(&x);
            acc
        });
    Ok((bin_size, mem_size, hex_string))
}

async fn build_wat(
    wat_path: &str,
    bin_size: usize,
    mem_size: usize,
    hex_string: String,
) -> Result<()> {
    let wat_content = format!(
        r#"(module (import "ethereum" "finish" (func (param i32 i32)))
      (func (export "main")
        i32.const 0
        i32.const {}
        call 0)
      (memory (export "memory") {})
      (data (i32.const 0) "{}"))
    "#,
        bin_size, mem_size, hex_string
    );
    write(wat_path, wat_content)
        .await
        .context("fail to write deploy wasm script")?;
    Ok(())
}

async fn build_deploy_wasm(wat_path: &str, wasm_path: &str) -> Result<()> {
    Command::new("wat2wasm")
        .args(&[wat_path, "-o", wasm_path])
        .output()
        .await
        .context("fail to build deploy wasm")?;
    Ok(())
}

async fn generate_deploy_wasm_hex(wasm_path: &str, text_path: &str) -> Result<()> {
    let bin = read(Path::new(wasm_path)).await?;
    let hex_string = encode(bin);
    write(text_path, hex_string)
        .await
        .context("fail to generate hex string for deploy wasm")?;
    Ok(())
}

pub async fn run() -> Result<String> {
    let res = tokio::try_join!(check_dependency(), check_cargo_toml());

    let contract_name = match res {
        Ok((_, contract_name)) => contract_name,
        Err(err) => return Err(err),
    };
    build_runtime_wasm().await?;

    let (bin_size, mem_size, hex_string) = generate_runtime_info(format!(
        "./target/wasm32-unknown-unknown/release/{}.wasm",
        contract_name
    ))
    .await?;

    let wat_path = format!(
        "./target/wasm32-unknown-unknown/release/{}.wat",
        contract_name
    );
    let wasm_path = format!(deploy_wasm!(), contract_name);
    let text_path = format!(deploy_file!(), contract_name);

    build_wat(&wat_path, bin_size, mem_size, hex_string).await?;
    build_deploy_wasm(&wat_path, &wasm_path).await?;
    generate_deploy_wasm_hex(&wasm_path, &text_path).await?;
    Ok(contract_name)
}
