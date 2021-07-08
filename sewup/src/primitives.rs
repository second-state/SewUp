#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

use super::errors::ContractError::ContractSizeError;

use anyhow::Result;

pub type FunctionSignature = [u8; 4];

pub struct Contract {
    pub data_size: usize,
    pub input_data: Vec<u8>,
    fn_sig: FunctionSignature,
}

impl Contract {
    pub fn new() -> Result<Self> {
        #[cfg(target_arch = "wasm32")]
        let data_size = ewasm_api::calldata_size();
        #[cfg(not(target_arch = "wasm32"))]
        let data_size = 0;

        #[cfg(target_arch = "wasm32")]
        let input_data = ewasm_api::calldata_acquire();
        #[cfg(not(target_arch = "wasm32"))]
        let input_data = Vec::new();

        #[cfg(target_arch = "wasm32")]
        let fn_sig = input_data[0..4]
            .try_into()
            .expect("size greator than 4 after validated");
        #[cfg(not(target_arch = "wasm32"))]
        let fn_sig = [0, 0, 0, 0];

        if data_size < 4 {
            Err(ContractSizeError(data_size).into())
        } else {
            Ok(Contract {
                data_size,
                input_data,
                fn_sig,
            })
        }
    }
    pub fn get_function_selector(&self) -> Result<FunctionSignature> {
        Ok(self.fn_sig)
    }
}
