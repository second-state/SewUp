use std::env;
use std::path::Path;

use anyhow::Result;
use structopt::StructOpt;
use tokio;

mod build;
mod deploy;
mod generate;
mod init;
mod inspect;

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

    /// Debug mode, generate hexstring format for deploy wasm
    #[structopt(short, long)]
    debug: bool,

    /// Inspect the .deploy file to wat
    #[structopt(short, long)]
    inspect_file: Option<String>,

    /// Generate ABI JSON if the handler is compaitabled with web3.js
    #[structopt(short, long)]
    generate_abi: bool,

    /// `init` sub command to init project on current folder or on `--project_path`
    sub_command: Option<String>,

    // The seconde argument will be the real sub_command if user call the cargo-sewup from cargo
    second_argument: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    if let Some(path) = opt.project_path {
        let path = Path::new(&path);
        if tokio::fs::metadata(path).await.is_err() {
            tokio::fs::create_dir_all(path).await?;
        }
        env::set_current_dir(&path)?;
    }
    if opt.verbose {
        println!("project   : {}", env::current_dir()?.display());
    }

    if let Some(sub_command) = opt.sub_command {
        if sub_command == "init"
            || (sub_command == "sewup" && opt.second_argument == Some("init".into()))
        {
            init::run().await
        } else {
            println!("Unknown sub command {:?}", sub_command);
            Ok(())
        }
    } else {
        return if let Some(inspect_file) = opt.inspect_file {
            inspect::run(inspect_file).await
        } else if opt.generate_abi {
            generate::run().await
        } else {
            let contract_name = build::run(opt.debug).await?;

            if !opt.build_only {
                if opt.verbose {
                    println!("contract  : {}", contract_name);
                }
                deploy::run(contract_name, opt.verbose, opt.debug).await?;
            }
            Ok(())
        };
    }
}
