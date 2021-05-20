use crate::utils::*;
use ewasm_api::types::*;

pub fn do_transfer(recipient: Address, value: StorageValue) {
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

pub fn do_balance(account: Address) {
    let balance = get_balance(&account);

    if balance.bytes != StorageValue::default().bytes {
        ewasm_api::finish_data(&balance.bytes);
    }
}

pub fn name() {
    let token_name = "ERC20TokenDemo".to_string().into_bytes();
    ewasm_api::finish_data(&token_name);
}

pub fn symbol() {
    let symbol = "ETD".to_string().into_bytes();
    ewasm_api::finish_data(&symbol);
}

pub fn decimals() {
    let decimals = 0_u64.to_be_bytes();
    ewasm_api::finish_data(&decimals);
}

pub fn total_supply() {
    let total_supply = 100000000_u64.to_be_bytes();
    ewasm_api::finish_data(&total_supply);
}

pub fn approve(spender: Address, value: StorageValue) {
    let sender = ewasm_api::caller();

    set_allowance(&sender, &spender, &value);
}

pub fn allowance(owner: Address, spender: Address) {
    let allowance_value = get_allowance(&owner, &spender);

    ewasm_api::finish_data(&allowance_value.bytes);
}

pub fn mint(the: Address, value: u64) {
    let value: [u8; 8] = value.to_be_bytes();
    let stv_owner_balance = copy_into_storage_value(&value[0..8]);
    set_balance(&the, &stv_owner_balance);
}

pub fn transfer_from(owner: Address, recipient: Address, value: u64) {
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
