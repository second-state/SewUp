pub use crate::utils::{copy_into_array, sha3_256};

#[cfg(target_arch = "wasm32")]
use ewasm_api::types::{Address, StorageKey, StorageValue};

#[cfg(not(target_arch = "wasm32"))]
pub struct Address {}
#[cfg(not(target_arch = "wasm32"))]
pub struct StorageValue {}

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

#[cfg(not(target_arch = "wasm32"))]
pub fn get_balance(_address: &Address) -> StorageValue {
    StorageValue {}
}
#[cfg(target_arch = "wasm32")]
pub fn get_balance(address: &Address) -> StorageValue {
    let hash = calculate_balance_hash(&address.bytes);

    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_load(&storage_key)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_balance(_address: &Address, _value: &StorageValue) {}
#[cfg(target_arch = "wasm32")]
pub fn set_balance(address: &Address, value: &StorageValue) {
    let hash = calculate_balance_hash(&address.bytes);
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
pub fn copy_into_storage_value(_slice: &[u8]) -> StorageValue {
    StorageValue {}
}
#[cfg(target_arch = "wasm32")]
pub fn copy_into_storage_value(slice: &[u8]) -> StorageValue {
    let mut sk = StorageKey::default();
    sk.bytes[24..32].copy_from_slice(slice);
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
