use std::iter::FromIterator;

use anyhow::{Context, Result};
use regex::Regex;
use tokio::{
    fs::{create_dir_all, read_to_string, write},
    process::Command,
};

pub async fn run() -> Result<()> {
    let builder = tempfile::Builder::new();
    let outdir = builder.tempdir().expect("failed to create tmp file");
    let outfile_path = outdir.path().join("expanded");
    let generator_proj_src = outdir.path().join("g").join("src");
    create_dir_all(&generator_proj_src).await?;
    let toml_path = outdir.path().join("g").join("Cargo.toml");
    let abi_generator_path = generator_proj_src.join("main.rs");

    Command::new("cargo")
        .args(&[
            "rustc",
            "--target=wasm32-unknown-unknown",
            "--",
            "-o",
            outfile_path.to_str().unwrap(),
            "-Zunpretty=expanded",
        ])
        .output()
        .await
        .context("fail to expand macro")?;
    let expanded = read_to_string(outfile_path).await?;

    let sig_re = Regex::new(r"[A-Za-z0-9:_]*_SIG").unwrap();
    let abi_re =
        Regex::new(r#"const (?P<abi_name>[A-Za-z0-9_]*_ABI): &'static str =[^;]*;"#).unwrap();
    let total_abis: Vec<String> = sig_re
        .find_iter(&expanded)
        .map(|m| m.as_str().replace("_SIG", "_ABI"))
        .collect();
    let contract_abis = abi_re.captures_iter(&expanded).map(|c| {
        (
            c.name("abi_name").unwrap().as_str().to_string(),
            c.get(0).unwrap().as_str().to_string(),
        )
    });

    let all_abis = linked_hash_set::LinkedHashSet::<String>::from_iter(total_abis.iter().cloned());
    let mut lib_abis = all_abis.clone();
    let mut contract_abi_context = String::new();
    for (abi, def) in contract_abis.into_iter() {
        lib_abis.remove(&abi);
        contract_abi_context = contract_abi_context + &def;
    }

    write(
        toml_path.to_str().unwrap(),
        r#"
        [package]
        name = "g"
        version = "0.1.0"
        edition = "2018"
        [dependencies]
        sewup = { version = "*", features=["kv", "rdb", "token"] }
    "#,
    )
    .await?;

    let generator = format!(
        r#"
            {}
            fn main () {{
                println!(
                        "[{}]"
                        , {}
                );
            }}
        "#,
        contract_abi_context,
        vec!["{}"; all_abis.len()].join(","),
        all_abis.into_iter().collect::<Vec<String>>().join(",")
    );

    write(abi_generator_path.to_str().unwrap(), generator).await?;

    let output = Command::new("cargo")
        .args(&[
            "run",
            &format!("--manifest-path={}", toml_path.to_str().unwrap()),
        ])
        .output()
        .await
        .context("fail to compile generator")?;

    let abi_json = String::from_utf8_lossy(&output.stdout);

    println!("{}", abi_json);
    Ok(())
}
