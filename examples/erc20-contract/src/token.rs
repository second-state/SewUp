use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[cfg(target_arch = "wasm32")]
use ewasm_api::types::*;

#[ewasm_fn]
fn do_balance(contract: &sewup::primitives::Contract) {
    if contract.data_size != 24 {
        ewasm_api::revert();
    }
    let address_data = contract.input_data[4..].to_vec();
    let address = sewup::token::helpers::copy_into_address(&address_data[0..20]);
    sewup::token::erc20::do_balance(address);
}

#[ewasm_fn]
fn do_transfer(contract: &sewup::primitives::Contract) {
    if contract.input_data.len() != 32 {
        ewasm_api::revert();
    }

    let recipient_data = contract.input_data[4..24].to_vec();
    let recipient = sewup::token::helpers::copy_into_address(&recipient_data[0..20]);

    let value_data: [u8; 8] = sewup::token::helpers::copy_into_array(&contract.input_data[24..]);
    let mut value = StorageValue::default();
    let value_len = value_data.len();
    let start = 32 - value_len;

    value.bytes[start..(value_len + start)]
        .clone_from_slice(&value_data[..((value_len + start) - start)]);

    sewup::token::erc20::do_transfer(recipient, value);
}

#[ewasm_fn]
fn approve(contract: &sewup::primitives::Contract) {
    let spender_data = contract.input_data[4..24].to_vec();
    let spender = sewup::token::helpers::copy_into_address(&spender_data[0..20]);

    let value = contract.input_data[24..32].to_vec();
    let storage_value = sewup::token::helpers::copy_into_storage_value(&value[0..8]);
    sewup::token::erc20::approve(spender, storage_value);
}

#[ewasm_fn]
fn allowance(contract: &sewup::primitives::Contract) {
    if contract.data_size != 44 {
        ewasm_api::revert();
    }

    let from_data = contract.input_data[4..24].to_vec();
    let from = sewup::token::helpers::copy_into_address(&from_data[0..20]);

    let spender_data = contract.input_data[24..44].to_vec();
    let spender = sewup::token::helpers::copy_into_address(&spender_data[0..20]);

    sewup::token::erc20::allowance(from, spender);
}

#[ewasm_fn]
fn transfer_from(contract: &sewup::primitives::Contract) {
    if contract.data_size != 52 {
        ewasm_api::revert();
    }

    let owner = sewup::token::helpers::copy_into_address(&contract.input_data[4..24]);

    let recipient = sewup::token::helpers::copy_into_address(&contract.input_data[24..44]);

    let value_data: [u8; 8] = sewup::token::helpers::copy_into_array(&contract.input_data[44..52]);

    let value = u64::from_be_bytes(value_data);

    sewup::token::erc20::transfer_from(owner, recipient, value);
}

#[ewasm_fn]
fn mint(contract: &sewup::primitives::Contract) {
    let address = sewup::token::helpers::copy_into_address(&contract.input_data[4..24]);

    let value_data: [u8; 8] = sewup::token::helpers::copy_into_array(&contract.input_data[24..32]);
    let value = u64::from_be_bytes(value_data);

    sewup::token::erc20::mint(address, value);
}

#[ewasm_main]
fn main() -> anyhow::Result<()> {
    let contract = sewup::primitives::Contract::new()?;
    match contract.get_function_selector()? {
        ewasm_fn_sig!(do_balance) => do_balance(&contract),
        ewasm_fn_sig!(do_transfer) => do_transfer(&contract),
        sewup::token::erc20::NAME_SIG => sewup::token::erc20::name(),
        sewup::token::erc20::SYMBOL_SIG => sewup::token::erc20::symbol("ETD"),
        sewup::token::erc20::DECIMALS_SIG => sewup::token::erc20::decimals(),
        sewup::token::erc20::TOTAL_SUPPLY_SIG => sewup::token::erc20::total_supply(),
        ewasm_fn_sig!(approve) => approve(&contract),
        ewasm_fn_sig!(allowance) => allowance(&contract),
        ewasm_fn_sig!(transfer_from) => transfer_from(&contract),
        ewasm_fn_sig!(mint) => mint(&contract),
        _ => (),
    };
    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use hex_literal::hex;
    use sewup::erc20::{DECIMALS_SIG, NAME_SIG, SYMBOL_SIG, TOTAL_SUPPLY_SIG};
    use sewup_derive::ewasm_assert_eq;

    #[ewasm_test]
    fn test_execute_basic_operations() {
        ewasm_assert_eq!(
            name(),
            vec![69, 82, 67, 50, 48, 84, 111, 107, 101, 110, 68, 101, 109, 111,]
        );
        ewasm_assert_eq!(symbol(), vec![69, 84, 68]);
        ewasm_assert_eq!(decimals(), vec![0, 0, 0, 0, 0, 0, 0, 0]);
        ewasm_assert_eq!(total_supply(), vec![0, 0, 0, 0, 5, 245, 225, 0]);
        let balance_input = hex!("00000000000000000000000000000000FACEB00C");
        ewasm_assert_eq!(do_balance(balance_input), vec![]);
    }
}
