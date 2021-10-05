use std::path::Path;

use anyhow::{Context, Result};
use clap::arg_enum;
use tokio::fs::{create_dir, write};

use cargo_sewup::{
    constants::{AUTO_CONTRACT, DEFAULT_CONTRACT, RUSTY_CONTRACT},
    default_cargo_template, rusty_cargo_template,
};

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
        "./.cargo/config",
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

    let cargo_content = match mode {
        Mode::Rusty => format!(rusty_cargo_template!(), project_name, project_name),
        _ => format!(default_cargo_template!(), project_name, project_name),
    };
    write("Cargo.toml", cargo_content)
        .await
        .context("failed to init Cargo.toml")?;
    Ok(())
}

async fn init_lib_file(mode: &Mode) -> Result<()> {
    create_dir(Path::new("./src"))
        .await
        .context("failed to create src folder")?;
    let task = match mode {
        Mode::Rusty => write("./src/lib.rs", RUSTY_CONTRACT),
        Mode::Auto => write("./src/lib.rs", AUTO_CONTRACT),
        Mode::Default => write("./src/lib.rs", DEFAULT_CONTRACT),
    };
    task.await.context("failed to create sample code")?;
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
