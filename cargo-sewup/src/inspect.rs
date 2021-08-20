use std::collections::HashSet;

use anyhow::{Context, Result};
use hex::decode;
use tokio::fs::read_to_string;
use wasmparser::{Parser, Payload};

pub async fn run(inspect_file: String) -> Result<()> {
    let mut deploy_content = read_to_string(inspect_file)
        .await
        .context("can not read the deploy file you want to inspect")?;

    deploy_content = deploy_content.trim_end().to_string();

    let mut exports = HashSet::<String>::new();

    let buf: Vec<u8> = decode(&deploy_content).context("fail to decode deploy file to binary")?;
    for payload in Parser::new(0).parse_all(&buf) {
        match payload? {
            Payload::Version { num, .. } => {
                println!("====== Module version: {}", num);
            }
            Payload::ExportSection(s) => {
                for export in s {
                    let export = export?;
                    println!("  Export {} {:?}", export.field, export.kind);
                    exports.insert(format!("{} {:?}", export.field, export.kind));
                }
            }
            Payload::ImportSection(s) => {
                for import in s {
                    let import = import?;
                    println!("  Import {}::{}", import.module, import.field.unwrap());
                }
            }
            _other => {
                println!("  {:?}", _other);
            }
        }
    }
    if exports.len() != 2
        || !exports.contains("memory Memory")
        || !exports.contains("main Function")
    {
        println!("====== Result");
        println!("Ethereum wasm only export memory and main function");
    }
    Ok(())
}
