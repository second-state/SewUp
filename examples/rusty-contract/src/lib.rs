use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup_derive::{ewasm_fn, ewasm_main, fn_sig, input_from};

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
