//! ERC20 contract handler
//! The handler helps you deploy/generate ERC20 token, just like manipulating a
//! database.
//!
//! The general way we connect to a DB is to use the connection string
//! postgres://user:password@127.0.0.1:5432
//!
//! This handler mimic the connecting way for ERC20 contract
//! erc20://sender_address@node_ip:node_port/ERC20_contract_config_file_path
//! erc20://sender_address@node_ip:node_port/contract_address

use anyhow::Result;
use ethereum_types::Address;
use ewasm_api;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

/// ERC20ContractHandler helps you deploy or interactive with the existing
/// ERC 20 contract
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ERC20ContractHandler {
    /// The contract address, when the contract is not deployed,
    /// the `connect` function called the contract will be automatically
    /// deployed and the address field of the config will be filled
    pub address: Option<Address>,

    #[serde(skip_serializing)]
    /// The contract data in hex literal for eWasm binary, or a file path to
    /// the .ewasm file
    pub call_data: Option<String>,

    /// If the ERC20_contract_config_file_path pass from the connection string,
    /// this field will be filled
    #[serde(skip)]
    config_file_path: Option<PathBuf>,
}

impl ERC20ContractHandler {
    pub fn connect(&mut self) -> Result<()> {
        if self.address.is_none() {
            return self._deploy();
        }
        Ok(())
    }

    fn _deploy(&mut self) -> Result<()> {
        let input_data = ewasm_api::calldata_acquire();
        // if data_size < 4 {
        //     ewasm_api::revert();
        // }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::H160;
    use toml;
    #[test]
    fn test_config_serde() {
        let c1 = ERC20ContractHandler {
            address: Some(H160::from_low_u64_be(15)),
            call_data: Some("0x12345678".into()),
            ..Default::default()
        };
        assert_eq!(
            toml::to_string(&c1).unwrap(),
            "address = \"0x000000000000000000000000000000000000000f\"\n"
        );

        let c2: ERC20ContractHandler =
            toml::from_str("address = \"0x000000000000000000000000000000000000000f\"\n").unwrap();
        assert_eq!(c2.address, c1.address);

        let c3: ERC20ContractHandler = toml::from_str("call_data = \"0x12345678\"\n").unwrap();
        assert_eq!(c3.address, None);
    }
}
