use std::path::Path;

use anyhow::{Context, Result};
use hex::encode;
use tokio::{
    fs::{read, read_to_string, write},
    process::Command,
};
use wat;

use crate::config::Toml;
use crate::deploy_wasm;

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

async fn build_wat(bin_size: usize, mem_size: usize, hex_string: String) -> Result<String> {
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
    Ok(wat_content)
}

async fn build_deploy_wasm(wat_content: String, wasm_path: &str) -> Result<()> {
    let binary = wat::parse_str(wat_content)?;
    write(wasm_path, binary).await?;
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

pub async fn run(debug: bool) -> Result<String> {
    let contract_name = check_cargo_toml().await?;
    build_runtime_wasm().await?;

    let (bin_size, mem_size, hex_string) = generate_runtime_info(format!(
        "./target/wasm32-unknown-unknown/release/{}.wasm",
        contract_name
    ))
    .await?;

    let wat_content = build_wat(bin_size, mem_size, hex_string).await?;

    let wasm_path = format!(deploy_wasm!(), contract_name);
    build_deploy_wasm(wat_content, &wasm_path).await?;

    if debug {
        let text_path = format!(
            "./target/wasm32-unknown-unknown/release/{}.deploy",
            contract_name
        );
        generate_deploy_wasm_hex(&wasm_path, &text_path).await?;
    }

    Ok(contract_name)
}
