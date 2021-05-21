use ewasm_api::types::*;

mod erc20;
mod signature;
mod utils;

use crate::erc20::*;
use crate::signature::*;
use crate::utils::*;

#[no_mangle]
pub fn main() {
    let data_size = ewasm_api::calldata_size();
    let input_data = ewasm_api::calldata_acquire();

    if data_size < 4 {
        ewasm_api::revert();
    }

    let function_selector = input_data[0..4].to_vec();

    if function_selector == DO_BALANCE_SIGNATURE {
        if data_size != 24 {
            ewasm_api::revert();
        }

        let address_data = input_data[4..].to_vec();
        let address = copy_into_address(&address_data[0..20]);
        do_balance(address)
    }

    if function_selector == DO_TRANSFER_SIGNATURE {
        if input_data.len() != 32 {
            ewasm_api::revert();
        }

        let recipient_data = input_data[4..24].to_vec();
        let recipient = copy_into_address(&recipient_data[0..20]);

        let value_data: [u8; 8] = copy_into_array(&input_data[24..]);
        let mut value = StorageValue::default();
        let value_len = value_data.len();
        let start = 32 - value_len;

        value.bytes[start..(value_len + start)]
            .clone_from_slice(&value_data[..((value_len + start) - start)]);

        do_transfer(recipient, value);
    }

    if function_selector == NAME_SIGNATURE {
        name();
    }

    if function_selector == SYMBOL_SIGNATURE {
        symbol();
    }

    if function_selector == DEVIMALS_SIGNATURE {
        decimals();
    }

    if function_selector == TOTAL_SUPPLY_SIGNATURE {
        total_supply();
    }

    if function_selector == APPROVE_SIGNATURE {
        let spender_data = input_data[4..24].to_vec();
        let spender = copy_into_address(&spender_data[0..20]);

        let value = input_data[24..32].to_vec();
        let storage_value = copy_into_storage_value(&value[0..8]);

        approve(spender, storage_value);
    }

    if function_selector == ALLOWANCE_SIGNATURE {
        if data_size != 44 {
            ewasm_api::revert();
        }

        let from_data = input_data[4..24].to_vec();
        let from = copy_into_address(&from_data[0..20]);

        let spender_data = input_data[24..44].to_vec();
        let spender = copy_into_address(&spender_data[0..20]);

        allowance(from, spender);
    }

    if function_selector == TRANSFER_FROM_SIGNATURE {
        if data_size != 52 {
            ewasm_api::revert();
        }

        let owner = copy_into_address(&input_data[4..24]);

        let recipient = copy_into_address(&input_data[24..44]);

        let value_data: [u8; 8] = copy_into_array(&input_data[44..52]);

        let value = u64::from_be_bytes(value_data);

        transfer_from(owner, recipient, value);
    }

    if function_selector == MINT_SIGNATURE {
        let adddress = copy_into_address(&input_data[4..24]);

        let value_data: [u8; 8] = copy_into_array(&input_data[24..32]);
        let value = u64::from_be_bytes(value_data);

        mint(adddress, value);
    }
}
