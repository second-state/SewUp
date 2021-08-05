use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[cfg(target_arch = "wasm32")]
use ewasm_api::types::*;

#[ewasm_constructor]
fn constructor() {}

#[ewasm_main]
fn main() -> anyhow::Result<()> {
    let contract = sewup::primitives::Contract::new()?;
    match contract.get_function_selector()? {
        sewup::token::erc20::BALANCE_OF_SIG => sewup::token::erc20::balance_of(&contract),
        sewup::token::erc20::TRANSFER_SIG => sewup::token::erc20::transfer(&contract),
        sewup::token::erc20::NAME_SIG => sewup::token::erc20::name("Demo"),
        sewup::token::erc20::SYMBOL_SIG => sewup::token::erc20::symbol("ETD"),
        sewup::token::erc20::DECIMALS_SIG => sewup::token::erc20::decimals(),
        sewup::token::erc20::TOTAL_SUPPLY_SIG => sewup::token::erc20::total_supply(),
        sewup::token::erc20::APPROVE_SIG => sewup::token::erc20::approve(&contract),
        sewup::token::erc20::ALLOWANCE_SIG => sewup::token::erc20::allowance(&contract),
        sewup::token::erc20::TRANSFER_FROM_SIG => sewup::token::erc20::transfer_from(&contract),
        _ => (),
    };
    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use hex_literal::hex;
    use sewup::erc20::{BALANCE_OF_SIG, DECIMALS_SIG, NAME_SIG, SYMBOL_SIG, TOTAL_SUPPLY_SIG};
    use sewup_derive::ewasm_assert_eq;

    #[ewasm_test]
    fn test_execute_basic_operations() {
        ewasm_assert_eq!(name(), vec![68, 101, 109, 111]);
        ewasm_assert_eq!(symbol(), vec![69, 84, 68]);
        ewasm_assert_eq!(decimals(), vec![0, 0, 0, 0, 0, 0, 0, 0]);
        ewasm_assert_eq!(total_supply(), vec![0, 0, 0, 0, 5, 245, 225, 0]);
        let balance_input = hex!("00000000000000000000000000000000FACEB00C");
        ewasm_assert_eq!(balance_of(balance_input), vec![]);
    }
}
