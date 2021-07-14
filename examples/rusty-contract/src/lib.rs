use serde_derive::{Deserialize, Serialize};
use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[derive(Default, Serialize, Deserialize)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[ewasm_fn]
fn check_input_object(s: SimpleStruct) -> Result<(), &'static str> {
    if !s.trust {
        return Err("NotTrustedInput");
    }
    Ok(())
}

#[ewasm_main(rusty)]
fn main() -> Result<(), &'static str> {
    use sewup::primitives::Contract;
    use sewup_derive::ewasm_input_from;

    let contract = Contract::new().map_err(|_| "NewContractError")?;
    match contract
        .get_function_selector()
        .map_err(|_| "FailGetFnSelector")?
    {
        ewasm_fn_sig!(check_input_object) => {
            ewasm_input_from!(contract move check_input_object, |_| "DeserdeError")?
        }
        _ => return Err("UnknownHandle"),
    };

    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup::primitives::Contract;
    use sewup_derive::{ewasm_assert_eq, ewasm_rusty_assert_ok, ewasm_rusty_err_output};

    #[ewasm_test]
    fn test_execute_rusty_contract() {
        let mut simple_struct = SimpleStruct::default();

        // You can easily use any kind of rust error like this
        ewasm_assert_eq!(
            check_input_object(simple_struct),
            ewasm_rusty_err_output!(Err("NotTrustedInput") as Result<(), &'static str>)
        );

        // Assert an error result (default is thiserror) with raw output,
        // the previous `ewasm_assert_eq` is the suggested way
        ewasm_assert_eq!(
            check_input_object(simple_struct),
            vec![
                1, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 78, 111, 116, 84, 114, 117, 115, 116, 101,
                100, 73, 110, 112, 117, 116
            ]
        );

        simple_struct.trust = true;

        // use `ewasm_assert_rusty_ok`, because the `#[ewasm_main(rusty)]` specify the rusty return
        ewasm_rusty_assert_ok!(check_input_object(simple_struct));

        // Assert an ok result with raw output,
        // the previous `ewasm_assert_rusty_ok` is the suggested way
        ewasm_assert_eq!(check_input_object(simple_struct), vec![0, 0, 0, 0]);
    }
}
