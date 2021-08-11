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
use bitcoin::util::uint::Uint256;
#[cfg(target_arch = "wasm32")]
use ewasm_api::types::{Address, StorageValue};
#[cfg(target_arch = "wasm32")]
use hex::decode;

#[cfg(not(target_arch = "wasm32"))]
use super::helpers::{Address, StorageValue};

use sewup_derive::ewasm_lib_fn;

/// Implement ERC-20 transfer(address,uint256)
/// ```json
/// {
///     "constant": false,
///     "inputs": [
///         { "internalType": "address", "name": "recipient", "type": "address" },
///         { "internalType": "uint256", "name": "amount", "type": "uint256" }
///     ],
///     "name": "transfer",
///     "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
///     "payable": false,
///     "stateMutability": "nonpayable",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn(a9059cbb)]
pub fn transfer(contract: &Contract) {
    let sender = ewasm_api::caller();
    let recipient = copy_into_address(&contract.input_data[16..36]);

    let value = {
        let value_data: [u8; 32] = copy_into_array(&contract.input_data[36..68]);
        Uint256::from_be_bytes(value_data)
    };

    let sender_storage_value = {
        let balance = get_balance(&sender);
        let origin_value = Uint256::from_be_bytes(balance.bytes);
        let new_value = origin_value - value;
        let buffer = new_value.to_be_bytes();
        copy_into_storage_value(&buffer)
    };

    let recipient_storage_value = {
        let balance = get_balance(&recipient);
        let origin_value = Uint256::from_be_bytes(balance.bytes);
        let new_value = origin_value + value;
        let buffer = new_value.to_be_bytes();
        copy_into_storage_value(&buffer)
    };

    set_balance(&sender, &sender_storage_value);
    set_balance(&recipient, &recipient_storage_value);
}

/// Implement ERC-20 balanceOf(address)
/// ```json
/// {
///     "constant": true,
///     "inputs": [{ "internalType": "address", "name": "account", "type": "address" }],
///     "name": "balanceOf",
///     "outputs": [{ "internalType": "uint256", "name": "", "type": "uint256" }],
///     "payable": false,
///     "stateMutability": "view",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn(70a08231)]
pub fn balance_of(contract: &Contract) {
    let address = copy_into_address(&contract.input_data[16..36]);
    let balance = get_balance(&address);
    ewasm_api::finish_data(&balance.bytes);
}

/// Implement ERC-20 name() and easy to change the name
/// ```json
/// {
///     "constant": true,
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
/// {
///     "constant": true,
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
///     "outputs": [{ "internalType": "uint256", "name": "", "type": "uint8" }],
///     "payable": false, "stateMutability": "view", "type": "function"
/// }
/// ```
#[ewasm_lib_fn(313ce567)]
pub fn decimals(i: u8) {
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

#[cfg(target_arch = "wasm32")]
pub fn mint(addr: &str, value: usize) {
    let byte20: [u8; 20] = decode(addr)
        .expect("address should be hex format")
        .try_into()
        .expect("address should be byte20");
    set_balance(
        &Address::from(byte20),
        &Raw::from(value).to_bytes32().into(),
    );
}
