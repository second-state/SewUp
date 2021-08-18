use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {}

#[ewasm_main]
fn main() -> anyhow::Result<()> {
    let contract = sewup::primitives::Contract::new()?;
    match contract.get_function_selector()? {
        sewup::token::erc1155::BALANCE_OF_SIG => sewup::token::erc1155::balance_of(&contract),
        sewup::token::erc1155::BALANCE_OF_BATCH_SIG => {
            sewup::token::erc1155::balance_of_batch(&contract)
        }
        _ => (),
    };
    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
}
