use std::convert::TryInto;

use crate::primitives::Contract;
use crate::types::Raw;

#[cfg(target_arch = "wasm32")]
use super::helpers::{
    copy_into_address, copy_into_array, copy_into_storage_value, get_allowance, get_balance,
    set_allowance, set_balance,
};

#[cfg(target_arch = "wasm32")]
use crate::utils::ewasm_return_str;
#[cfg(target_arch = "wasm32")]
use ewasm_api::types::{Address, StorageValue};

#[cfg(not(target_arch = "wasm32"))]
use super::helpers::{Address, StorageValue};

use sewup_derive::ewasm_lib_fn;

/// Implement ERC-20 transfer(address,uint256)
#[ewasm_lib_fn(5d359fbd)]
pub fn transfer(contract: &Contract) {
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

    let sender = ewasm_api::caller();
    let sender_balance = get_balance(&sender);

    let recipient_balance = get_balance(&recipient);

    let sb_bytes: [u8; 8] = copy_into_array(&sender_balance.bytes[24..32]);
    let sb_u64 = u64::from_be_bytes(sb_bytes);

    let val_bytes: [u8; 8] = copy_into_array(&value.bytes[24..32]);
    let val_u64 = u64::from_be_bytes(val_bytes);

    let new_sb_u64 = sb_u64 - val_u64;

    let new_sb_bytes: [u8; 8] = new_sb_u64.to_be_bytes();
    let sb_value = copy_into_storage_value(&new_sb_bytes[0..8]);

    let rc_bytes: [u8; 8] = copy_into_array(&recipient_balance.bytes[24..32]);
    let rc_u64 = u64::from_be_bytes(rc_bytes);

    let new_rc_u64 = rc_u64 + val_u64;

    let new_rc_bytes: [u8; 8] = new_rc_u64.to_be_bytes();
    let rc_value = copy_into_storage_value(&new_rc_bytes[0..8]);

    set_balance(&sender, &sb_value);
    set_balance(&recipient, &rc_value);
}

/// Implement ERC-20 balanceOf(address)
#[ewasm_lib_fn(70a08231)]
pub fn balance_of(contract: &Contract) {
    if contract.data_size != 24 {
        ewasm_api::revert();
    }
    let address_data = contract.input_data[4..].to_vec();
    let address = copy_into_address(&address_data[0..20]);

    let balance = get_balance(&address);

    if balance.bytes != StorageValue::default().bytes {
        ewasm_api::finish_data(&balance.bytes);
    }
}

/// Implement ERC-20 name() and easy to change the name
/// ```json
/// { "constant": true,
///     "inputs": [],
///     "name": "symbol",
///     "outputs": [{ "internalType": "string", "name": "", "type": "string" }],
///     "payable": false, "stateMutability": "view",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn(06fdde03)]
pub fn name(s: &str) {
    ewasm_return_str(s);
}

/// Implement ERC-20 symbol() and easy to change the symbol
/// ```json
/// { "constant": true,
///     "inputs": [],
///     "name": "symbol",
///     "outputs": [{ "internalType": "string", "name": "", "type": "string" }],
///     "payable": false, "stateMutability": "view",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn(95d89b41)]
pub fn symbol(s: &str) {
    ewasm_return_str(s);
}

/// Implement ERC-20 decimals()
/// ```json
/// {
///     "constant": true,
///     "inputs": [],
///     "name": "decimals",
///     "outputs": [{ "internalType": "uint256", "name": "", "type": "uint256" }],
///     "payable": false, "stateMutability": "view", "type": "function"
/// }
/// ```
#[ewasm_lib_fn(313ce567)]
pub fn decimals(i: usize) {
    ewasm_api::finish_data(&Raw::from(i).as_bytes().to_vec());
}

/// Implement ERC-20 totalSupply()
/// ```json
/// {
///     "constant": true,
///     "inputs": [],
///     "name": "totalSupply",
///     "outputs": [{ "internalType": "uint256", "name": "", "type": "uint256" }],
///     "payable": false, "stateMutability": "view", "type": "function"
/// }
/// ```
#[ewasm_lib_fn(18160ddd)]
pub fn total_supply(i: usize) {
    ewasm_api::finish_data(&Raw::from(i).as_bytes().to_vec());
}

/// Implement ERC-20 approve(address,uint256)
#[ewasm_lib_fn("095ea7b3")]
pub fn approve(contract: &Contract) {
    let spender_data = contract.input_data[4..24].to_vec();
    let spender = copy_into_address(&spender_data[0..20]);

    let value = contract.input_data[24..56].to_vec();
    let storage_value = copy_into_storage_value(&value[0..8]);

    let sender = ewasm_api::caller();
    let byte32: [u8; 32] = value.try_into().expect("value should be byte32");

    set_allowance(&sender, &spender, &byte32.into());
}

/// Implement ERC-20 allowance(address,address)
#[ewasm_lib_fn(dd62ed3e)]
pub fn allowance(contract: &Contract) {
    if contract.data_size != 44 {
        ewasm_api::revert();
    }

    let from_data = contract.input_data[4..24].to_vec();
    let owner = copy_into_address(&from_data[0..20]);

    let spender_data = contract.input_data[24..44].to_vec();
    let spender = copy_into_address(&spender_data[0..20]);

    let allowance_value = get_allowance(&owner, &spender);

    ewasm_api::finish_data(&allowance_value.bytes);
}

/// Implement ERC-20 transferFrom(address,address,uint256)
#[ewasm_lib_fn(23b872dd)]
pub fn transfer_from(contract: &Contract) {
    if contract.data_size != 52 {
        ewasm_api::revert();
    }

    let owner = copy_into_address(&contract.input_data[4..24]);

    let recipient = copy_into_address(&contract.input_data[24..44]);

    let value_data: [u8; 8] = copy_into_array(&contract.input_data[44..52]);

    let value = u64::from_be_bytes(value_data);

    let sender = ewasm_api::caller();
    let owner_balance = get_balance(&owner);

    let ob_bytes: [u8; 8] = copy_into_array(&owner_balance.bytes[24..32]);
    let mut owner_balance = u64::from_be_bytes(ob_bytes);

    if owner_balance < value {
        ewasm_api::revert();
    }

    let allowed_value = get_allowance(&owner, &sender);

    let a_bytes: [u8; 8] = copy_into_array(&allowed_value.bytes[24..32]);
    let mut allowed = u64::from_be_bytes(a_bytes);

    if value > allowed {
        ewasm_api::revert();
    }

    let recipient_balance = get_balance(&recipient);

    let rb_bytes: [u8; 8] = copy_into_array(&recipient_balance.bytes[24..32]);
    let mut recipient_balance = u64::from_be_bytes(rb_bytes);

    owner_balance -= value;
    recipient_balance += value;
    allowed -= value;

    let owner_balance_bytes: [u8; 8] = owner_balance.to_be_bytes();
    let stv_owner_balance = copy_into_storage_value(&owner_balance_bytes[0..8]);

    let recipient_balance_bytes: [u8; 8] = recipient_balance.to_be_bytes();
    let stv_recipient_balance = copy_into_storage_value(&recipient_balance_bytes[0..8]);

    let allowed_bytes: [u8; 8] = allowed.to_be_bytes();
    let stv_allowed = copy_into_storage_value(&allowed_bytes[0..8]);

    set_balance(&owner, &stv_owner_balance);
    set_balance(&recipient, &stv_recipient_balance);
    set_allowance(&owner, &sender, &stv_allowed);
}
