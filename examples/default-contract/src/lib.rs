use serde_derive::{Deserialize, Serialize};
use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

mod errors;
use errors::Error;

#[derive(Default, Serialize, Deserialize)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[cfg(feature = "constructor")]
#[no_mangle]
fn constructor() {
    let a = 1;
    let b = 2;
    let c = a + b;
    sewup::utils::ewasm_return(vec![1, 2, c, 4]);
}

#[cfg(not(feature = "constructor"))]
#[ewasm_fn]
fn check_input_object(s: SimpleStruct) -> anyhow::Result<()> {
    // ewasm_dbg! help you debug things when this ewasm is running by testruntime
    // To show the debug message pllease run the test case as following command
    // `cargo test -- --nocapture`
    // Or you may checkout the log file set by following `ewasm_test` macro
    // `#[ewasm_test(log=/tmp/default.log)]`
    if !sewup::ewasm_dbg!(s.trust) {
        return Err(Error::NotTrustedInput.into());
    }
    Ok(())
}

#[cfg(not(feature = "constructor"))]
#[ewasm_main]
fn main() -> anyhow::Result<()> {
    use sewup::primitives::Contract;
    use sewup_derive::ewasm_input_from;

    let contract = Contract::new()?;
    match contract.get_function_selector()? {
        ewasm_fn_sig!(check_input_object) => ewasm_input_from!(contract move check_input_object)?,
        _ => return Err(Error::UnknownHandle.into()),
    };

    Ok(())
}

#[ewasm_test(log=/tmp/default.log)]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_err_output};

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
