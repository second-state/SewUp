//! ERC20 contract handler
//! The handler helps you deploy/generate ERC20 token, just like manipulating a
//! database.
//!
//! The general way we connect to a DB is to use the connection string
//! > postgres://user:password@127.0.0.1:5432
//!
//! This handler mimic the database connecting way for ERC20 contract, and
//! automatically deploy the contract if needed
//! >>>
//! erc20://sender_address@node_ip:node_port/ERC20_contract_config_file_path
//! erc20://sender_address@node_ip:node_port/contract_address
//! >>>

use crate::token::errors::ContractError as Error;
use anyhow::Result;
use ethereum_types::Address;
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
    pub(crate) config_file_path: Option<PathBuf>,
}

impl ERC20ContractHandler {
    /// Reach the contract, if the contract is not exist, then deploy it first
    pub fn connect(&mut self) -> Result<()> {
        if self.address.is_none() {
            return self.deploy();
        }
        Ok(())
    }

    /// deploy the contract from call data store contract address into config
    fn deploy(&mut self) -> Result<()> {
        if let Some(_config_file_path) = self.config_file_path.take() {
            if let Some(call_data) = self.call_data.take() {
                if let Some(call_data) = ERC20ContractHandler::get_call_data(call_data) {
                    if call_data.len() < 4 {
                        return Err(Error::ContractSizeError(call_data.len()).into());
                    }
                    return Ok(());
                }
            }
            return Err(Error::InsufficientContractInfoError.into());
        }
        panic!("config_file_path should be update when ERC20ContractHandler construct")
    }
    /// Return the call data binary from hex literal or from a ewasm file
    fn get_call_data(_call_data_info: String) -> Option<Vec<u8>> {
        Some(vec![0, 0])
    }
}
