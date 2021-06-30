use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup_derive::{
    ewasm_assert_eq, ewasm_fn, ewasm_main, ewasm_test, fn_sig, input_from, output_from,
};

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

#[ewasm_test]
mod tests {
    use super::*;

    #[ewasm_test]
    fn test_execute_basic_operations() {
        let mut simple_struct = SimpleStruct::default();

        ewasm_assert_eq!(
            check_input_object(simple_struct),
            ewasm_err_output!(Error::NotTrustedInput)
        );

        // Assert an error result (default is thiserror) with raw output,
        // the previous `ewasm_assert_eq` is the suggested way
        ewasm_assert_eq!(
            check_input_object(simple_struct),
            vec![110, 111, 116, 32, 116, 114, 117, 115, 116, 32, 105, 110, 112, 117, 116,]
        );

        simple_struct.trust = true;

        ewasm_assert_ok!(check_input_object(simple_struct));

        // Assert an ok result with raw output,
        // the previous `ewasm_assert_ok` is the suggested way
        ewasm_assert_eq!(check_input_object(simple_struct), vec![]);
    }
}
