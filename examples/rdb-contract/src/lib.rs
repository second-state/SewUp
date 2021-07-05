use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup::types::{Raw, Row};
use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_output_from, ewasm_test, Table};

mod errors;
use errors::RDBError;

#[derive(Table)]
struct Person {
    trusted: bool,
    age: u8,
}

#[ewasm_main]
fn main() -> Result<()> {
    let contract = Contract::new()?;

    match contract.get_function_selector()? {
        ewasm_fn_sig!(person::get) => person::get(),
        ewasm_fn_sig!(person::create) => person::create(),
        ewasm_fn_sig!(person::update) => person::update(),
        ewasm_fn_sig!(person::delete) => person::delete(),
        _ => return Err(RDBError::UnknownHandle.into()),
    };

    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_err_output};

    #[ewasm_test]
    fn test_execute_crud_handler() {
        // TODO: correctly implement the handler
        ewasm_assert_eq!(person::get(), vec![]);
        ewasm_assert_eq!(person::create(), vec![]);
        ewasm_assert_eq!(person::update(), vec![]);
        ewasm_assert_eq!(person::delete(), vec![]);
    }
}
