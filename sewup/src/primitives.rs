use std::convert::TryInto;

use super::errors::ContractError::ContractSizeError;

use anyhow::Result;

pub type FunctionSignature = [u8; 4];

pub struct Contract {
    pub data_size: usize,
    pub input_data: Vec<u8>,
}

impl Contract {
    pub fn new() -> Result<Self> {
        let data_size = ewasm_api::calldata_size();
        let input_data = ewasm_api::calldata_acquire();
        if data_size < 4 {
            Err(ContractSizeError(data_size).into())
        } else {
            Ok(Contract {
                data_size,
                input_data,
            })
        }
    }
    pub fn get_function_selector(&self) -> Result<FunctionSignature> {
        Ok(self.input_data[0..4]
            .try_into()
            .expect("size greator than 4 after validated"))
    }
}
