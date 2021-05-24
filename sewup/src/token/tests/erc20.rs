//! ERC20 contract handler
//! The handler helps you deploy/generate ERC20 token, just like manipulating a
//! database.
//!
//! The general way we connect to a DB is to use the connection string
//! > postgres://user:password@127.0.0.1:5432
//!
//! This handler mimic the database connecting way for ERC20 contract, and
//! automatically deploy the contract if needed
//!
//!
//! ## Getting started
//!
//! Add follow sewup with `token` feature enabled.
//! > sewup = { features = ["token"] }
//!
//! >>>
//! sewup://sender_address@node_ip:node_port/ERC20_contract_config_file_path
//! sewup://sender_address@node_ip:node_port/erc20_contract_address
//! >>>

use std::cell::RefCell;
use std::fmt;
use std::fs::{self, read};
use std::path::PathBuf;
use std::sync::Arc;

use crate::errors::ContractError as Error;
use crate::runtimes::traits::{VMMessageBuilder, RT};

use anyhow::{Context, Result};
use ethereum_types::Address;
use serde_derive::{Deserialize, Serialize};

/// ERC20ContractHandler helps you deploy or interactive with the existing
/// ERC 20 contract
#[derive(Clone, Deserialize, Serialize, Default)]
pub struct ERC20ContractHandler {
    /// The contract address, when the contract is not deployed,
    /// the `connect` function called the contract will be automatically
    /// deployed and the address field of the config will be filled
    pub contract_address: Option<Address>,

    /// The address of the sender, who calls the contract
    #[serde(skip)]
    pub sender_address: Address,

    /// The contract data in hex literal for eWasm binary, or a file path to
    /// the .ewasm file
    pub call_data: Option<String>,

    /// If the ERC20_contract_config_file_path pass from the connection string,
    /// this field will be filled
    #[serde(skip)]
    pub(crate) config_file_path: Option<PathBuf>,

    #[serde(skip)]
    pub(crate) rt: Option<Arc<RefCell<dyn RT>>>,
}

impl fmt::Debug for ERC20ContractHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
            .field("contract_address", &self.contract_address)
            .field("sender_address", &self.sender_address)
            .field("call_data", &self.call_data)
            .field("config_file_path", &self.config_file_path)
            .field("rt", &self.rt.is_some())
            .finish()
    }
}

impl ERC20ContractHandler {
    /// Reach the contract, if the contract is not exist, then deploy it first
    pub fn connect(&mut self, gas: i64) -> Result<()> {
        if self.contract_address.is_none() {
            return self.deploy(gas);
        }
        Ok(())
    }

    /// deploy the contract from call data store contract address into config
    fn deploy(&mut self, gas: i64) -> Result<()> {
        if let Some(config_file_path) = self.config_file_path.take() {
            if let Some(call_data) = self.call_data.take() {
                let call_data = ERC20ContractHandler::get_call_data(call_data)?;
                if call_data.len() < 4 {
                    return Err(Error::ContractSizeError(call_data.len()).into());
                }
                if let Some(rt) = self.rt.take() {
                    let msg = VMMessageBuilder {
                        sender: Some(&self.sender_address),
                        input_data: Some(&call_data),
                        gas,
                        ..Default::default()
                    }
                    .build()?;
                    self.contract_address = Some(*rt.borrow_mut().deploy(msg)?);
                    self.rt = Some(rt);
                    fs::write(
                        config_file_path,
                        toml::to_string(self).expect("config generate error"),
                    )
                    .expect("config file can not be updated");
                    return Ok(());
                }
                panic!("rt should be init when parsing the connection string")
            }
            return Err(Error::InsufficientContractInfoError.into());
        }
        panic!("config_file_path should be update when ERC20ContractHandler construct")
    }

    /// Return the call data binary from hex literal or from a ewasm file
    fn get_call_data(call_data_info: String) -> Result<Vec<u8>> {
        if call_data_info.starts_with("0x") {
            if call_data_info.len() % 2 != 0 {
                return Err(Error::CalldataMalformat.into());
            }
            let mut format_error = false;
            let v = call_data_info[2..]
                .chars()
                .collect::<Vec<char>>()
                .chunks(2)
                .enumerate()
                .map(|(i, c)| {
                    u8::from_str_radix(c.iter().collect::<String>().as_str(), 16)
                        .with_context(|| {
                            format_error = true;
                            format!("Failed to parse call data at {}", i * 2 + 2)
                        })
                        .unwrap_or(0)
                })
                .collect::<Vec<u8>>();
            if format_error {
                return Err(Error::CalldataMalformat.into());
            }
            Ok(v)
        } else {
            Ok(read(call_data_info)?)
        }
    }
}
