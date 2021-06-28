//! Contract handler
//! The handler helps you deploy contract and test the contract you developing
use std::cell::RefCell;
use std::fmt;
use std::fs::{self, read};
use std::path::PathBuf;
use std::sync::Arc;

use crate::errors::ContractError as Error;
use crate::runtimes::traits::{VMMessageBuilder, VMResult, RT};

use anyhow::{Context, Result};
use ethereum_types::Address;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Default)]
pub struct ContractHandler {
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

    /// If the contract_config_file_path pass from the connection string,
    /// this field will be filled
    #[serde(skip)]
    pub config_file_path: Option<PathBuf>,

    #[serde(skip)]
    pub rt: Option<Arc<RefCell<dyn RT>>>,
}

impl fmt::Debug for ContractHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContractHandler")
            .field("contract_address", &self.contract_address)
            .field("sender_address", &self.sender_address)
            .field("call_data", &self.call_data)
            .field("config_file_path", &self.config_file_path)
            .field("rt", &self.rt.is_some())
            .finish()
    }
}

impl ContractHandler {
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
                let call_data = ContractHandler::get_call_data(call_data)?;
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
        panic!("config_file_path should be update when ContractHandler constructed")
    }

    pub fn execute(
        &mut self,
        fun_sig: [u8; 4],
        input: Option<&[u8]>,
        gas: i64,
    ) -> Result<VMResult> {
        if let Some(rt) = self.rt.take() {
            let mut result: Result<VMResult> = Err(Error::CalldataAbsent.into());
            if let Some(call_data) = self.call_data.take() {
                let call_data = ContractHandler::get_call_data(call_data)?;
                let mut input_data: Vec<u8> = fun_sig.to_vec();
                if let Some(input) = input {
                    input_data.extend_from_slice(input);
                }

                let msg = VMMessageBuilder {
                    sender: Some(&self.sender_address),
                    input_data: Some(&input_data),
                    gas,
                    code: Some(&call_data),
                    ..Default::default()
                }
                .build()?;
                result = Ok(rt.borrow_mut().execute(msg)?);
            }
            self.rt = Some(rt);
            return result;
        }
        panic!("rt should be init when parsing the connection string")
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
