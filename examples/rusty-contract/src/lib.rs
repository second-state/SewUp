use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup_derive::{ewasm_assert_eq, ewasm_fn, ewasm_main, ewasm_test, fn_sig, input_from};

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
    let contract = Contract::new().map_err(|_| "NewContractError")?;
    match contract
        .get_function_selector()
        .map_err(|_| "FailGetFnSelector")?
    {
        fn_sig!(check_input_object) => {
            input_from!(contract, check_input_object, |_| "DeserdeError")
                .map_err(|_| "InputError")?
        }
        _ => return Err("UnknownHandle"),
    };

    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;

    #[ewasm_test]
    fn test_execute_rusty_contract() {
        let mut simple_struct = SimpleStruct::default();

        ewasm_assert_eq!(
            check_input_object(simple_struct),
            vec![
                1, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 73, 110, 112, 117, 116, 69, 114, 114, 111,
                114,
            ]
        );

        simple_struct.trust = true;

        ewasm_assert_eq!(check_input_object(simple_struct), vec![0, 0, 0, 0]);
    }
}
