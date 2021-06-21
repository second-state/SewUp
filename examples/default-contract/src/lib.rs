use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup_derive::{ewasm_fn, ewasm_main, fn_sig, input_from};

mod errors;
use errors::Error;

#[derive(Default, Serialize, Deserialize)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[ewasm_fn]
fn check_input_object(s: SimpleStruct) -> Result<()> {
    if !s.trust {
        return Err(Error::NotTrustedInput.into());
    }
    Ok(())
}

#[ewasm_main]
fn main() -> Result<()> {
    let contract = Contract::new()?;
    match contract.get_function_selector()? {
        fn_sig!(check_input_object) => input_from!(contract, check_input_object)?,
        _ => return Err(Error::UnknownHandle.into()),
    };

    Ok(())
}
