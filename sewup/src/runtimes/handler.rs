//! Contract handler
//! The handler helps you deploy contract and test the contract you developing
use std::cell::RefCell;
use std::convert::TryInto;
use std::fs::read;
use std::sync::Arc;

use crate::errors::ContractError as Error;
use crate::runtimes::traits::{VMMessageBuilder, VMResult, RT};
use crate::types::Raw;

use anyhow::{Context, Result};
use hex::decode;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Default)]
pub struct ContractHandler {
    /// The contract data in hex literal for eWasm binary, or a file path to
    /// the .ewasm file
    pub call_data: Option<String>,
    #[serde(skip)]
    pub rt: Option<Arc<RefCell<dyn RT>>>,
}

#[cfg(any(feature = "debug", test))]
impl std::fmt::Debug for ContractHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractHandler")
            .field("call_data", &self.call_data)
            .field("rt", &self.rt.is_some())
            .finish()
    }
}

impl ContractHandler {
    /// run the call data as function directly, this function is suppose to be used for constructor
    pub fn run_fn(
        &mut self,
        call_data: String,
        input: Option<&[u8]>,
        gas: i64,
    ) -> Result<VMResult> {
        if let Some(rt) = self.rt.take() {
            let call_data = ContractHandler::get_call_data(call_data)?;
            let mut input_data: Vec<u8> = Vec::new();
            if let Some(input) = input {
                input_data.extend_from_slice(input);
            }
            let sender = Raw::default();
            let msg = VMMessageBuilder {
                sender: Some(&sender),
                input_data: Some(&input_data),
                gas,
                code: Some(&call_data),
                ..Default::default()
            }
            .build()?;
            let result = Ok(rt.borrow_mut().execute(msg)?);
            self.rt = Some(rt);
            return result;
        }
        panic!("rt should be init when parsing the connection string")
    }

    pub fn execute(
        &mut self,
        addr: Option<&str>,
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

                let sender = if let Some(addr) = addr {
                    let hex_str: &str = if addr.starts_with("0x") {
                        &addr[2..addr.len()]
                    } else {
                        addr
                    };
                    let byte20: [u8; 20] = decode(hex_str)
                        .expect("contract caller's address should be hex format")
                        .try_into()
                        .expect("contract caller's address should be bytes20");
                    Raw::from_raw_address(&byte20)
                } else {
                    Raw::default()
                };

                let msg = VMMessageBuilder {
                    sender: Some(&sender),
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
        if let Some(stripped_data_info) = call_data_info.strip_prefix("0x") {
            if call_data_info.len() % 2 != 0 {
                return Err(Error::CalldataMalformat.into());
            }
            let mut format_error = false;
            let v = stripped_data_info
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
