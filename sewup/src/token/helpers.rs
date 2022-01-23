#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

#[cfg(target_arch = "wasm32")]
use ewasm_api::types::Address;

use crate::types::Address as SewUpAddress;
#[cfg(target_arch = "wasm32")]
use crate::types::Raw;
use crate::utils::sha3_256;

#[cfg(target_arch = "wasm32")]
use ewasm_api::types::{StorageKey, StorageValue};

#[cfg(not(target_arch = "wasm32"))]
pub struct StorageValue {}

#[cfg(not(target_arch = "wasm32"))]
pub struct Address {}

pub fn calculate_approval_hash(sender: &[u8; 20], spender: &[u8; 20]) -> Vec<u8> {
    let mut allowance: Vec<u8> = "approval".as_bytes().into();
    allowance.extend_from_slice(sender);
    allowance.extend_from_slice(spender);
    sha3_256(&allowance).to_vec()
}

pub fn calculate_token_approval_hash(token_id: &[u8; 32]) -> Vec<u8> {
    let mut token_approval: Vec<u8> = "token approval".as_bytes().into();
    token_approval.extend_from_slice(token_id);
    sha3_256(&token_approval).to_vec()
}

pub fn calculate_allowance_hash(sender: &[u8; 20], spender: &[u8; 20]) -> Vec<u8> {
    let mut allowance: Vec<u8> = "allowance".as_bytes().into();
    allowance.extend_from_slice(sender);
    allowance.extend_from_slice(spender);
    sha3_256(&allowance).to_vec()
}

pub fn calculate_balance_hash(address: &[u8; 20]) -> Vec<u8> {
    let mut balance_of: Vec<u8> = "balanceOf".as_bytes().into();
    balance_of.extend_from_slice(address);
    sha3_256(&balance_of).to_vec()
}

pub fn calculate_token_hash(token_id: &[u8; 32]) -> Vec<u8> {
    let mut token: Vec<u8> = "token_id".as_bytes().into();
    token.extend_from_slice(token_id);
    sha3_256(&token).to_vec()
}

pub fn calculate_token_balance_hash(address: &[u8; 20], token_id: &[u8; 32]) -> Vec<u8> {
    let mut balance_of: Vec<u8> = "balanceOf".as_bytes().into();
    balance_of.extend_from_slice(address);
    balance_of.extend_from_slice(token_id);
    sha3_256(&balance_of).to_vec()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_balance(_address: &SewUpAddress) -> StorageValue {
    StorageValue {}
}
#[cfg(target_arch = "wasm32")]
pub fn get_balance(address: &SewUpAddress) -> StorageValue {
    let hash = calculate_balance_hash(&address.inner.bytes);

    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_load(&storage_key)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_balance(_address: &SewUpAddress, _value: &StorageValue) {}
#[cfg(target_arch = "wasm32")]
pub fn set_balance(address: &SewUpAddress, value: &StorageValue) {
    let hash = calculate_balance_hash(&address.inner.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_store(&storage_key, &value);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_token_balance(_address: &SewUpAddress, _token_id: &[u8; 32]) -> StorageValue {
    StorageValue {}
}
#[cfg(target_arch = "wasm32")]
pub fn get_token_balance(address: &SewUpAddress, token_id: &[u8; 32]) -> StorageValue {
    let hash = calculate_token_balance_hash(&address.inner.bytes, token_id);

    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_load(&storage_key)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_token_balance(_address: &Address, _token_id: &[u8; 32], _value: &StorageValue) {}
#[cfg(target_arch = "wasm32")]
pub fn set_token_balance(address: &Address, token_id: &[u8; 32], value: &StorageValue) {
    let hash = calculate_token_balance_hash(&address.bytes, token_id);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_store(&storage_key, &value);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_allowance(_sender: &Address, _spender: &Address) -> StorageValue {
    StorageValue {}
}
#[cfg(target_arch = "wasm32")]
pub fn get_allowance(sender: &Address, spender: &Address) -> StorageValue {
    let hash = calculate_allowance_hash(&sender.bytes, &spender.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_load(&storage_key)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_allowance(_sender: &Address, _spender: &Address, _value: &StorageValue) {}
#[cfg(target_arch = "wasm32")]
pub fn set_allowance(sender: &Address, spender: &Address, value: &StorageValue) {
    let hash = calculate_allowance_hash(&sender.bytes, &spender.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_store(&storage_key, &value);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_token_approval(_token_id: &[u8; 32]) -> Address {
    Address {}
}
#[cfg(target_arch = "wasm32")]
pub fn get_token_approval(token_id: &[u8; 32]) -> Address {
    let hash = calculate_token_approval_hash(token_id);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);
    let buf: [u8; 20] = ewasm_api::storage_load(&storage_key).bytes[12..32]
        .try_into()
        .expect("");
    buf.into()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_token_approval(_token_id: &[u8; 32], _spender: &Address) {}
#[cfg(target_arch = "wasm32")]
pub fn set_token_approval(token_id: &[u8; 32], spender: &Address) {
    let hash = calculate_token_approval_hash(token_id);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);
    ewasm_api::storage_store(&storage_key, &Raw::from(spender).to_bytes32().into());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_approval(_sender: &Address, _spender: &Address) -> bool {
    true
}
#[cfg(target_arch = "wasm32")]
pub fn get_approval(sender: &Address, spender: &Address) -> bool {
    let hash = calculate_approval_hash(&sender.bytes, &spender.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);
    ewasm_api::storage_load(&storage_key).bytes[31] == 1
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_approval(_sender: &Address, _spender: &Address, _is_approved: bool) {}
#[cfg(target_arch = "wasm32")]
pub fn set_approval(sender: &Address, spender: &Address, is_approved: bool) {
    let hash = calculate_approval_hash(&sender.bytes, &spender.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);
    let mut storage_value = StorageKey::default();
    storage_value.bytes[31] = if is_approved { 1 } else { 0 };
    ewasm_api::storage_store(&storage_key, &storage_value);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_into_storage_value(_slice: &[u8]) -> StorageValue {
    StorageValue {}
}
#[cfg(target_arch = "wasm32")]
pub fn copy_into_storage_value(slice: &[u8]) -> StorageValue {
    let mut sk = StorageKey::default();
    sk.bytes[0..32].copy_from_slice(slice);
    sk
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_into_address(_slice: &[u8]) -> Address {
    Address {}
}
#[cfg(target_arch = "wasm32")]
pub fn copy_into_address(slice: &[u8]) -> Address {
    let mut a = Address::default();
    a.bytes.copy_from_slice(slice);
    a
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_token_owner(_token_id: &[u8; 32], _owner: &Address) {}
#[cfg(target_arch = "wasm32")]
pub fn set_token_owner(token_id: &[u8; 32], owner: &Address) {
    let storage_key = copy_into_storage_value(&calculate_token_hash(token_id));
    let value: StorageValue = Raw::from(owner).to_bytes32().into();
    ewasm_api::storage_store(&storage_key, &value);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_token_owner(_token_id: &[u8; 32]) -> Address {
    Address {}
}
#[cfg(target_arch = "wasm32")]
pub fn get_token_owner(token_id: &[u8; 32]) -> Address {
    let storage_key = copy_into_storage_value(&calculate_token_hash(token_id));
    let storage_value = ewasm_api::storage_load(&storage_key);
    let bytes20: [u8; 20] = storage_value.bytes[12..32]
        .try_into()
        .expect("address should be bytes20");
    bytes20.into()
}
