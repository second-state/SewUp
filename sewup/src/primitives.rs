use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

use super::errors::ContractError::ContractSizeError;

pub type FunctionSignature = [u8; 4];

pub struct Contract {
    pub data_size: usize,
    pub input_data: Vec<u8>,
    fn_sig: FunctionSignature,
}

impl Contract {
    pub fn new() -> anyhow::Result<Self> {
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
    pub fn get_function_selector(&self) -> anyhow::Result<FunctionSignature> {
        Ok(self.fn_sig)
    }
    pub fn mock() -> Self {
        let data_size = 5;
        let input_data = vec![0, 0, 0, 0, 0];
        let fn_sig = [0, 0, 0, 0];
        Contract {
            data_size,
            input_data,
            fn_sig,
        }
    }
}

/// helps you return different type of date in the contract handlers
/// The any serializable data can easy to become EwasmAny by following command
/// `EwasmAny::from(protocol)`
/// and the data will preserialized and store in the EwasmAny structure,
/// once the `ewasm_main` function try to return the instance of EwasmAny, the preserialized data
/// will be returned.
pub struct EwasmAny {
    pub bin: Vec<u8>,
}

impl EwasmAny {
    pub fn from<T: Serialize>(instance: T) -> Self {
        Self {
            bin: bincode::serialize(&instance).expect("The input should be serializable"),
        }
    }
}

impl<T> From<T> for EwasmAny
where
    T: Serialize,
{
    fn from(i: T) -> Self {
        Self {
            bin: bincode::serialize(&i).unwrap(),
        }
    }
}
