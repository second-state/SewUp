use anyhow::Result;
use sewup::primitives::Contract;
use sewup::token::{
    erc20::{
        allowance as erc20_allowance, approve as erc20_approve, decimals,
        do_balance as erc20_do_balance, do_transfer as erc20_do_transfer, mint as erc20_mint, name,
        symbol, total_supply, transfer_from as erc20_transfer_from, DECIMALS_SIG, NAME_SIG,
        SYMBOL_SIG, TOTAL_SUPPLY_SIG,
    },
    helpers::{copy_into_address, copy_into_array, copy_into_storage_value},
};
use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[cfg(target_arch = "wasm32")]
use ewasm_api::types::*;

#[ewasm_fn]
fn do_balance(contract: &Contract) {
    if contract.data_size != 24 {
        ewasm_api::revert();
    }
    let address_data = contract.input_data[4..].to_vec();
    let address = copy_into_address(&address_data[0..20]);

    erc20_do_balance(address);
}

#[ewasm_fn]
fn do_transfer(contract: &Contract) {
    if contract.input_data.len() != 32 {
        ewasm_api::revert();
    }

    let recipient_data = contract.input_data[4..24].to_vec();
    let recipient = copy_into_address(&recipient_data[0..20]);

    let value_data: [u8; 8] = copy_into_array(&contract.input_data[24..]);
    let mut value = StorageValue::default();
    let value_len = value_data.len();
    let start = 32 - value_len;

    value.bytes[start..(value_len + start)]
        .clone_from_slice(&value_data[..((value_len + start) - start)]);

    erc20_do_transfer(recipient, value);
}

#[ewasm_fn]
fn approve(contract: &Contract) {
    let spender_data = contract.input_data[4..24].to_vec();
    let spender = copy_into_address(&spender_data[0..20]);

    let value = contract.input_data[24..32].to_vec();
    let storage_value = copy_into_storage_value(&value[0..8]);

    erc20_approve(spender, storage_value);
}

#[ewasm_fn]
fn allowance(contract: &Contract) {
    if contract.data_size != 44 {
        ewasm_api::revert();
    }

    let from_data = contract.input_data[4..24].to_vec();
    let from = copy_into_address(&from_data[0..20]);

    let spender_data = contract.input_data[24..44].to_vec();
    let spender = copy_into_address(&spender_data[0..20]);

    erc20_allowance(from, spender);
}

#[ewasm_fn]
fn transfer_from(contract: &Contract) {
    if contract.data_size != 52 {
        ewasm_api::revert();
    }

    let owner = copy_into_address(&contract.input_data[4..24]);

    let recipient = copy_into_address(&contract.input_data[24..44]);

    let value_data: [u8; 8] = copy_into_array(&contract.input_data[44..52]);

    let value = u64::from_be_bytes(value_data);

    erc20_transfer_from(owner, recipient, value);
}

#[ewasm_fn]
fn mint(contract: &Contract) {
    let adddress = copy_into_address(&contract.input_data[4..24]);

    let value_data: [u8; 8] = copy_into_array(&contract.input_data[24..32]);
    let value = u64::from_be_bytes(value_data);

    erc20_mint(adddress, value);
}

#[ewasm_main]
fn main() -> Result<()> {
    let contract = Contract::new()?;
    match contract.get_function_selector()? {
        ewasm_fn_sig!(do_balance) => do_balance(&contract),
        ewasm_fn_sig!(do_transfer) => do_transfer(&contract),
        NAME_SIG => name(),
        SYMBOL_SIG => symbol("ETD"),
        DECIMALS_SIG => decimals(),
        TOTAL_SUPPLY_SIG => total_supply(),
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
