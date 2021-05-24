pub use crate::utils::{copy_into_array, sha3_256};

use ewasm_api::types::{Address, StorageKey, StorageValue};

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

pub fn get_balance(address: &Address) -> StorageValue {
    let hash = calculate_balance_hash(&address.bytes);

    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_load(&storage_key)
}

pub fn set_balance(address: &Address, value: &StorageValue) {
    let hash = calculate_balance_hash(&address.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_store(&storage_key, &value);
}

pub fn get_allowance(sender: &Address, spender: &Address) -> StorageValue {
    let hash = calculate_allowance_hash(&sender.bytes, &spender.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_load(&storage_key)
}

pub fn set_allowance(sender: &Address, spender: &Address, value: &StorageValue) {
    let hash = calculate_allowance_hash(&sender.bytes, &spender.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_store(&storage_key, &value);
}

pub fn copy_into_storage_value(slice: &[u8]) -> StorageValue {
    let mut sk = StorageKey::default();
    sk.bytes[24..32].copy_from_slice(slice);
    sk
}

pub fn copy_into_address(slice: &[u8]) -> Address {
    let mut a = Address::default();
    a.bytes.copy_from_slice(slice);
    a
}
