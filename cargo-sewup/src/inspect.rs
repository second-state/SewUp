use std::env;

use anyhow::{Context, Result};
use hex::decode;
use tokio::{
    fs::{read_to_string, write},
    process::Command,
};

pub async fn run(inspect_file: String) -> Result<()> {
    let mut deploy_content = read_to_string(inspect_file)
        .await
        .context("can not read the deploy file you want to inspect")?;

    deploy_content = deploy_content.trim_end().to_string();

    let wasm_bin = decode(&deploy_content).context("fail to decode deploy file to binary")?;

    let temp_wasm_path = env::temp_dir().join("inspect.wasm");

    write(temp_wasm_path.clone(), wasm_bin)
        .await
        .context("fail to generate hex string for deploy wasm")?;

    let output = Command::new("wasm2wat")
        .args(&[temp_wasm_path])
        .output()
        .await
        .context("fail to build wasm")?;
    let tmpl = String::from_utf8_lossy(&output.stdout);

    println!("{}", tmpl);

    Ok(())
}
