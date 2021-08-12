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
use ewasm_api::{log3, types::Address};
#[cfg(target_arch = "wasm32")]
use hex::decode;

#[cfg(not(target_arch = "wasm32"))]
use super::helpers::Address;

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

    let topic: [u8; 32] =
        decode("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
            .unwrap()
            .try_into()
            .unwrap();
    log3(
        &Vec::<u8>::with_capacity(0),
        &topic.into(),
        &Raw::from(sender).to_bytes32().into(),
        &Raw::from(recipient).to_bytes32().into(),
    );
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
/// ```json
/// {
///     "constant": false,
///     "inputs": [
///         { "internalType": "address", "name": "spender", "type": "address" },
///         { "internalType": "uint256", "name": "value", "type": "uint256" }
///     ],
///     "name": "approve",
///     "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
///     "payable": false,
///     "stateMutability": "nonpayable",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn("095ea7b3")]
pub fn approve(contract: &Contract) {
    let sender = ewasm_api::caller();
    let spender = copy_into_address(&contract.input_data[16..36]);
    let value = {
        let buffer: [u8; 32] = copy_into_array(&contract.input_data[36..68]);
        copy_into_storage_value(&buffer)
    };
    set_allowance(&sender, &spender, &value);
    let topic: [u8; 32] =
        decode("8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925")
            .unwrap()
            .try_into()
            .unwrap();
    log3(
        &Vec::<u8>::with_capacity(0),
        &topic.into(),
        &Raw::from(sender).to_bytes32().into(),
        &Raw::from(spender).to_bytes32().into(),
    );
}

/// Implement ERC-20 allowance(address,address)
/// ```json
/// {
///     "constant": true,
///     "inputs": [
///         { "internalType": "address", "name": "owner", "type": "address" },
///         { "internalType": "address", "name": "spender", "type": "address" }
///     ],
///     "name": "allowance",
///     "outputs": [{ "internalType": "uint256", "name": "", "type": "uint256" }],
///     "payable": false,
///     "stateMutability": "view",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn(dd62ed3e)]
pub fn allowance(contract: &Contract) {
    let owner = copy_into_address(&contract.input_data[16..36]);
    let spender = copy_into_address(&contract.input_data[48..68]);
    let allowance_value = get_allowance(&owner, &spender);
    ewasm_api::finish_data(&allowance_value.bytes);
}

/// Implement ERC-20 transferFrom(address,address,uint256)
/// ```json
/// {
///     "constant": false,
///     "inputs": [
///         { "internalType": "address", "name": "sender", "type": "address" },
///         { "internalType": "address", "name": "recipient", "type": "address" },
///         { "internalType": "uint256", "name": "amount", "type": "uint256" }
///     ],
///     "name": "transferFrom",
///     "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
///     "payable": false, "stateMutability": "nonpayable", "type": "function"
/// }
/// ```
#[ewasm_lib_fn(23b872dd)]
pub fn transfer_from(contract: &Contract) {
    let sender = ewasm_api::caller();
    let owner = copy_into_address(&contract.input_data[16..36]);
    let recipient = copy_into_address(&contract.input_data[48..68]);

    let amount = {
        let buffer: [u8; 32] = copy_into_array(&contract.input_data[68..100]);
        Uint256::from_be_bytes(buffer)
    };

    let mut allowed = {
        let allowed_value = get_allowance(&owner, &sender);
        Uint256::from_be_bytes(allowed_value.bytes)
    };

    let mut owner_balance = {
        let owner_balance = get_balance(&owner);
        Uint256::from_be_bytes(owner_balance.bytes)
    };

    let mut recipient_balance = {
        let recipient_balance = get_balance(&recipient);
        Uint256::from_be_bytes(recipient_balance.bytes)
    };

    if owner_balance < amount || amount > allowed {
        ewasm_api::revert();
    }

    owner_balance = owner_balance - amount;
    recipient_balance = recipient_balance + amount;
    allowed = allowed - amount;

    let owner_storage_value = {
        let buffer = owner_balance.to_be_bytes();
        copy_into_storage_value(&buffer)
    };

    let recipient_storage_value = {
        let buffer = recipient_balance.to_be_bytes();
        copy_into_storage_value(&buffer)
    };

    let allowed_storage_value = {
        let buffer = allowed.to_be_bytes();
        copy_into_storage_value(&buffer)
    };

    set_balance(&owner, &owner_storage_value);
    set_balance(&recipient, &recipient_storage_value);
    set_allowance(&owner, &sender, &allowed_storage_value);
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
