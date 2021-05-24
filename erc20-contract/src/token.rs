use anyhow::Result;
use ewasm_api::types::*;
use sewup::primitives::Contract;
use sewup::token::{
    erc20::{
        allowance, approve, decimals, do_balance, do_transfer, mint, name, symbol, total_supply,
        transfer_from,
    },
    helpers::{copy_into_address, copy_into_array, copy_into_storage_value},
    signature::{
        ALLOWANCE_SIGNATURE, APPROVE_SIGNATURE, DEVIMALS_SIGNATURE, DO_BALANCE_SIGNATURE,
        DO_TRANSFER_SIGNATURE, MINT_SIGNATURE, NAME_SIGNATURE, SYMBOL_SIGNATURE,
        TOTAL_SUPPLY_SIGNATURE, TRANSFER_FROM_SIGNATURE,
    },
};

#[no_mangle]
pub fn main() {
    fn inner() -> Result<()> {
        let contract = Contract::new()?;
        let function_selector = contract.get_function_selector()?;

        if function_selector == DO_BALANCE_SIGNATURE {
            if contract.data_size != 24 {
                ewasm_api::revert();
            }

            let address_data = contract.input_data[4..].to_vec();
            let address = copy_into_address(&address_data[0..20]);
            do_balance(address)
        }

        if function_selector == DO_TRANSFER_SIGNATURE {
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
            let spender_data = contract.input_data[4..24].to_vec();
            let spender = copy_into_address(&spender_data[0..20]);

            let value = contract.input_data[24..32].to_vec();
            let storage_value = copy_into_storage_value(&value[0..8]);

            approve(spender, storage_value);
        }

        if function_selector == ALLOWANCE_SIGNATURE {
            if contract.data_size != 44 {
                ewasm_api::revert();
            }

            let from_data = contract.input_data[4..24].to_vec();
            let from = copy_into_address(&from_data[0..20]);

            let spender_data = contract.input_data[24..44].to_vec();
            let spender = copy_into_address(&spender_data[0..20]);

            allowance(from, spender);
        }

        if function_selector == TRANSFER_FROM_SIGNATURE {
            if contract.data_size != 52 {
                ewasm_api::revert();
            }

            let owner = copy_into_address(&contract.input_data[4..24]);

            let recipient = copy_into_address(&contract.input_data[24..44]);

            let value_data: [u8; 8] = copy_into_array(&contract.input_data[44..52]);

            let value = u64::from_be_bytes(value_data);

            transfer_from(owner, recipient, value);
        }

        if function_selector == MINT_SIGNATURE {
            let adddress = copy_into_address(&contract.input_data[4..24]);

            let value_data: [u8; 8] = copy_into_array(&contract.input_data[24..32]);
            let value = u64::from_be_bytes(value_data);

            mint(adddress, value);
        }
        Ok(())
    }

    if let Err(_e) = inner() {
        // println!("{:?}", e);
    }
}
