use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup::types::{Raw, Row};
use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

mod errors;
use errors::RDBError;

// #[derive(EwasmTable)]
struct Person {
    trusted: bool,
    age: u8,
}

#[ewasm_main]
fn main() -> Result<()> {
    let contract = Contract::new()?;

    match contract.get_function_selector()? {
        _ => return Err(RDBError::UnknownHandle.into()),
    };

    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_err_output};
}
