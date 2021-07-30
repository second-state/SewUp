use std::fmt::{Display, Error, Formatter};

use serde_derive::Deserialize;

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
}

#[derive(Deserialize)]
pub struct Toml {
    pub package: Package,
    pub deploy: Option<Deploy>,
}
