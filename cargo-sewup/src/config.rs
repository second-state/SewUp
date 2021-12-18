use std::fmt::{Display, Error, Formatter};

use anyhow::{Context, Result};
use serde_derive::Deserialize;
use tokio::fs::read_to_string;

use crate::constants::{DEFAULT_GAS, DEFAULT_GAS_PRICE};

#[derive(Deserialize)]
pub struct Deploy {
    pub url: String,
    pub private: String,
    pub address: String,
    pub gas: Option<usize>,
    pub gas_price: Option<usize>,
}

impl Display for Deploy {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "url       : {}", self.url)?;
        writeln!(f, "private   : {}", self.private)?;
        writeln!(f, "address   : {}", self.address)?;
        writeln!(f, "gas       : {}", self.gas.unwrap_or(DEFAULT_GAS))?;
        writeln!(
            f,
            "gas price : {}",
            self.gas_price.unwrap_or(DEFAULT_GAS_PRICE)
        )?;
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize)]
pub struct CargoToml {
    pub package: Package,
}

#[derive(Deserialize)]
pub struct CargoLock {
    pub package: Vec<Package>,
}

#[derive(Deserialize)]
pub struct DeployToml {
    pub deploy: Deploy,
}

pub async fn get_deploy_config() -> Result<Deploy> {
    let config_contents = read_to_string("sewup.toml")
        .await
        .context("can not read sewup.toml")?;
    let config: DeployToml = toml::from_str(config_contents.as_str())?;

    Ok(config.deploy)
}
