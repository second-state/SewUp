use std::env;
use std::path::Path;

use anyhow::Result;
use structopt::StructOpt;
use tokio;

mod build;
mod config;
mod constants;
mod deploy;
mod errors;

#[derive(StructOpt)]
#[structopt(name = "cargo-sewup")]
struct Opt {
    /// Build deploy wasm only
    #[structopt(short, long)]
    build_only: bool,

    /// Path to sewup project, or current directory as default
    #[structopt(short, long)]
    project_path: Option<String>,

    /// Verbose mode
    #[structopt(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    if let Some(path) = opt.project_path {
        env::set_current_dir(&Path::new(&path))?
    }

    if opt.verbose {
        println!("project   : {}", env::current_dir()?.display());
    }

    let contract_name = build::run().await?;

    if !opt.build_only {
        if opt.verbose {
            println!("contract  : {}", contract_name);
        }
        deploy::run(contract_name, opt.verbose).await?;
    }

    Ok(())
}
