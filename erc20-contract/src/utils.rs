use ewasm_api::types::*;
use tiny_keccak::{Hasher, Sha3};

pub fn calculate_allowance_hash(sender: &[u8; 20], spender: &[u8; 20]) -> Vec<u8> {
    let mut allowance: Vec<u8> = vec![97, 108, 108, 111, 119, 97, 110, 99, 101]; // "allowance"
    allowance.extend_from_slice(sender);
    allowance.extend_from_slice(spender);

    let mut output = [0; 32];
    let mut hasher = Sha3::v256();
    hasher.update(&allowance);
    hasher.finalize(&mut output);
    output.to_vec()
}

pub fn calculate_balance_hash(address: &[u8; 20]) -> Vec<u8> {
    let mut balance_of: Vec<u8> = vec![98, 97, 108, 97, 110, 99, 101, 79, 102]; // "balanceOf"
    balance_of.extend_from_slice(address);

    let mut output = [0; 32];
    let mut hasher = Sha3::v256();
    hasher.update(&balance_of);
    hasher.finalize(&mut output);
    output.to_vec()
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

use std::convert::AsMut;

pub fn copy_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Copy,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
    a
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
