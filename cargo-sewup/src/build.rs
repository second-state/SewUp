use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result};
use hex::encode;
use regex::Regex;
use tokio::{
    fs::{read, read_to_string, write},
    process::Command,
};
use wat;

use crate::config::Toml;
use crate::deploy_wasm;

async fn check_dependency() -> Result<()> {
    Command::new("wasm2wat")
        .stderr(Stdio::null())
        .output()
        .await
        .context("wasm2wat not found")?;
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

async fn build_constructor_template(contract_wasm_path: &str) -> Result<String> {
    Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target=wasm32-unknown-unknown",
            "--features=constructor",
        ])
        .output()
        .await
        .context("fail to build runtime wasm")?;
    let output = Command::new("wasm2wat")
        .args(&[contract_wasm_path])
        .output()
        .await
        .context("fail to build runtime wasm")?;
    let tmpl = String::from_utf8_lossy(&output.stdout);
    Ok(tmpl.to_string())
}

async fn build_runtime_wasm() -> Result<()> {
    Command::new("cargo")
        .args(&["build", "--release", "--target=wasm32-unknown-unknown"])
        .output()
        .await
        .context("fail to build runtime wasm")?;
    Ok(())
}

async fn generate_runtime_info(rt_wasm_path: &str) -> Result<(usize, usize, String)> {
    let bin = read(Path::new(rt_wasm_path))
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
    tmpl: String,
    bin_size: usize,
    mem_size: usize,
    hex_string: String,
) -> Result<String> {
    let eth_finish_re =
        Regex::new(r#"\(import "ethereum" "finish" \(func (?P<eth_finish_sig>[^\s]*) "#).unwrap();
    let eth_finish_sig = if let Some(cap) = eth_finish_re.captures(&tmpl) {
        Some(cap.name("eth_finish_sig").unwrap().as_str())
    } else {
        None
    };

    let memory_re = Regex::new(r#"\(memory \(;0;\) (?P<mem_size>\d*)"#).unwrap();
    let mem_size = if let Some(cap) = memory_re.captures(&tmpl) {
        cap.name("mem_size")
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap()
            + mem_size
    } else {
        mem_size
    };

    let mut content = memory_re
        .replace(&tmpl, &format!(r#"(memory (;0;) {}"#, mem_size))
        .trim_end()
        .to_string();

    content.truncate(content.len() - 1);

    let export_re = Regex::new(r#"\(export.*\)\)\n"#).unwrap();
    content = export_re.replace_all(&content, "").trim_end().to_string();

    let main_call = if let Some(eth_finish_sig) = eth_finish_sig {
        format!(
            r#"
  (export "memory" (memory 0))
  (func (export "main") call $constructor i32.const 0 i32.const {} call {})"#,
            bin_size, eth_finish_sig
        )
    } else {
        let module_re = Regex::new(r#"\n\s*\(func \$"#).unwrap();
        content = module_re
            .replace(
                &content,
                r#"
  (import "ethereum" "finish" (func $$_Eth_Finish (param i32 i32)))
  (func $$"#,
            )
            .trim_end()
            .to_string();
        format!(
            r#"
  (export "memory" (memory 0))
  (func (export "main") call $constructor i32.const 0 i32.const {} call $_Eth_Finish)
        "#,
            bin_size
        )
    };
    content.push_str(&main_call);
    let data_section = format!(
        r#"
  (data (i32.const 0) "{}")"#,
        hex_string
    );
    content.push_str(&data_section);
    content.push(')');

    Ok(content)
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

async fn generate_debug_wat(wat_path: &str, wat_content: &str) -> Result<()> {
    write(wat_path, wat_content)
        .await
        .context("fail to write deploy wat")?;
    Ok(())
}

pub async fn run(debug: bool) -> Result<String> {
    let res = tokio::try_join!(check_dependency(), check_cargo_toml());
    let contract_name = match res {
        Ok((_, contract_name)) => contract_name,
        Err(err) => return Err(err),
    };

    let mut wasm_path = format!(
        "./target/wasm32-unknown-unknown/release/{}.wasm",
        contract_name
    );

    let wasm_tmpl = build_constructor_template(&wasm_path).await?;

    if debug {
        let tmpl_path = format!(
            "./target/wasm32-unknown-unknown/release/{}.tmpl.wat",
            contract_name
        );
        generate_debug_wat(&tmpl_path, &wasm_tmpl).await?;
    }

    build_runtime_wasm().await?;

    let (bin_size, mem_size, hex_string) = generate_runtime_info(&wasm_path).await?;

    let wat_content = build_wat(wasm_tmpl, bin_size, mem_size, hex_string).await?;

    if debug {
        let wat_path = format!(
            "./target/wasm32-unknown-unknown/release/{}.wat",
            contract_name
        );
        generate_debug_wat(&wat_path, &wat_content).await?;
    }

    wasm_path = format!(deploy_wasm!(), contract_name);
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
