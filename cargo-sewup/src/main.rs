use std::fs::{self, read, File};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Result;
use hex::encode;
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Package {
    pub name: String,
}

#[derive(Deserialize)]
struct Toml {
    pub package: Package,
}

fn check_dependency() -> Result<()> {
    if let Err(e) = Command::new("wat2wasm").stderr(Stdio::null()).spawn() {
        if ErrorKind::NotFound == e.kind() {
            println!("`wat2wasm` was not found! currently we use wat2wasm to generate deployer such that it make a deployable wasm");
        }
        Err(e.into())
    } else {
        Ok(())
    }
}

fn check_cargo_toml() -> Result<String> {
    let config_contents = fs::read_to_string("Cargo.toml")?;
    let config: Toml = toml::from_str(config_contents.as_str())?;

    // TODO: more toml config checking here

    Ok(config.package.name.replace("-", "_"))
}

fn build_runtime_wasm() -> Result<()> {
    Command::new("cargo")
        .args(&["build", "--release", "--target=wasm32-unknown-unknown"])
        .spawn()?;
    Ok(())
}

fn generate_runtime_info(rt_wasm_path: String) -> Result<(usize, usize, String)> {
    let bin = read(Path::new(&rt_wasm_path))?;
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

fn build_wat(wat_path: &str, bin_size: usize, mem_size: usize, hex_string: String) -> Result<()> {
    let mut wat_file = File::create(wat_path)?;
    write!(
        wat_file,
        r#"(module (import "ethereum" "finish" (func (param i32 i32)))
      (func (export "main")
        i32.const 0
        i32.const {}
        call 0)
      (memory (export "memory") {})
      (data (i32.const 0) "{}"))
    "#,
        bin_size, mem_size, hex_string
    )?;
    Ok(())
}

fn build_deploy_wasm(wat_path: &str, wasm_path: &str) -> Result<()> {
    Command::new("wat2wasm")
        .args(&[wat_path, "-o", wasm_path])
        .spawn()?;
    Ok(())
}

fn generate_deploy_wasm_hex(wasm_path: &str, text_path: &str) -> Result<()> {
    let bin = read(Path::new(wasm_path))?;
    let hex_string = encode(bin);
    fs::write(text_path, hex_string)?;
    Ok(())
}

fn main() -> Result<()> {
    check_dependency()?;
    let contract_name = check_cargo_toml()?;
    build_runtime_wasm()?;
    let (bin_size, mem_size, hex_string) = generate_runtime_info(format!(
        "./target/wasm32-unknown-unknown/release/{}.wasm",
        contract_name
    ))?;

    let wat_path = format!(
        "./target/wasm32-unknown-unknown/release/{}.wat",
        contract_name
    );
    let wasm_path = format!(
        "./target/wasm32-unknown-unknown/release/{}.deploy.wasm",
        contract_name
    );
    let text_path = format!(
        "./target/wasm32-unknown-unknown/release/{}.deploy",
        contract_name
    );

    build_wat(&wat_path, bin_size, mem_size, hex_string)?;
    build_deploy_wasm(&wat_path, &wasm_path)?;
    generate_deploy_wasm_hex(&wasm_path, &text_path)?;
    Ok(())
}
