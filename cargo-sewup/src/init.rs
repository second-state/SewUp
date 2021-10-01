use std::path::Path;

use anyhow::{Context, Result};
use clap::arg_enum;
use tokio::fs::{create_dir, write};

arg_enum! {
    pub enum Mode {
        Default,
        Rusty,
        Auto
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Default
    }
}

async fn init_gitignore() -> Result<()> {
    write(".gitignore", b"/target\n/sewup.toml")
        .await
        .context("failed to init .gitignore")?;
    Ok(())
}

async fn init_sewup_config() -> Result<()> {
    write(
        "sewup.toml",
        r#"
[deploy]
url = "http://localhost:8545"
private = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
address = "0xXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
gas = 5000000
gas_price = 1"#,
    )
    .await
    .context("failed to init sewup.toml")?;
    Ok(())
}
async fn init_cargo_config() -> Result<()> {
    create_dir(Path::new("./.cargo"))
        .await
        .context("failed to create .cargo folder")?;
    write(
        "./.cargo/config.rs",
        r#"
[target.'cfg(target_arch="wasm32")']
rustflags = ["-C", "link-arg=--export-table"]"#,
    )
    .await
    .context("failed to create cargo config")?;
    Ok(())
}

async fn init_cargo_toml(mode: &Mode) -> Result<()> {
    let current_folder = std::env::current_dir()?;
    let project_name = current_folder.file_name().unwrap().to_string_lossy();

    write(
        "Cargo.toml",
        format!(
            r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2018"

[lib]
path = "src/lib.rs"
crate-type = ["cdylib"]

# See the hello example if you want to use a rust client
# https://github.com/second-state/SewUp/tree/main/examples/hello-contract

[dependencies]
sewup = "*"
sewup-derive = "*"
anyhow = "*"

[profile.release]
incremental = false
panic = "abort"
lto = true
opt-level = "z"

[profile.release.package.{}]
incremental = false
opt-level = "z"

[features]
constructor = []
constructor-test = []"#,
            project_name, project_name
        ),
    )
    .await
    .context("failed to init Cargo.toml")?;
    Ok(())
}

async fn init_lib_file(mode: &Mode) -> Result<()> {
    create_dir(Path::new("./src"))
        .await
        .context("failed to create src folder")?;
    write(
        "./src/lib.rs",
        r#"
use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {}

#[ewasm_fn]
fn hello() -> anyhow::Result<String> {
    Ok("hello world".to_string())
}

#[ewasm_main(auto)]
fn main() -> anyhow::Result<String> {
    let contract = sewup::primitives::Contract::new()?;
    let greeting = match contract.get_function_selector()? {
        ewasm_fn_sig!(hello) => hello()?,
        _ => panic!("unknown handle"),
    };
    Ok(greeting)
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_auto_assert_eq, ewasm_output_from};

    #[ewasm_test]
    fn test_get_greeting() {
        ewasm_auto_assert_eq!(hello(), "hello world".to_string());
    }
}"#,
    )
    .await
    .context("failed to create sample code")?;
    Ok(())
}

pub async fn run(mode: Mode) -> Result<()> {
    tokio::try_join!(
        init_gitignore(),
        init_sewup_config(),
        init_cargo_config(),
        init_cargo_toml(&mode),
        init_lib_file(&mode)
    )?;
    Ok(())
}
