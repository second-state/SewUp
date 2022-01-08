#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

use crate::primitives::Contract;
#[cfg(target_arch = "wasm32")]
use crate::types::Raw;
use sewup_derive::ewasm_lib_fn;

#[cfg(target_arch = "wasm32")]
use super::helpers::{
    copy_into_address, copy_into_storage_value, get_approval, get_token_balance, set_token_balance,
};

#[cfg(target_arch = "wasm32")]
use crate::utils::ewasm_return_vec;

#[cfg(target_arch = "wasm32")]
use bitcoin::util::uint::Uint256;

#[cfg(target_arch = "wasm32")]
use ewasm_api::{log4, types::Address};

#[cfg(target_arch = "wasm32")]
use hex::decode;

pub use super::erc721::{
    is_approved_for_all, set_approval_for_all, IS_APPROVED_FOR_ALL_ABI, IS_APPROVED_FOR_ALL_SIG,
    SET_APPROVAL_FOR_ALL_ABI, SET_APPROVAL_FOR_ALL_SIG,
};

/// Implement ERC-1155 balanceOf(address,uint256)
#[ewasm_lib_fn(00fdd58e,
    constant=true,
    inputs=[
        { "internalType": "address", "name": "account", "type": "address" },
        { "internalType": "uinit256", "name": "token_id", "type": "uinit256" }
    ],
    name=balanceOf,
    outputs=[{ "internalType": "uint256", "name": "", "type": "uint256" }],
    payable=false,
    stateMutability=view
)]
pub fn balance_of(contract: &Contract) {
    let address = copy_into_address(&contract.input_data[16..36]);
    let token_id: [u8; 32] = contract.input_data[36..68]
        .try_into()
        .expect("token id should be byte32");
    let balance = get_token_balance(&address, &token_id);
    ewasm_api::finish_data(&balance.bytes);
}

/// Implement ERC-1155 balanceOfBatch(address[],uint256[])
#[ewasm_lib_fn(4e1273f4,
    constant=true,
    inputs=[
        { "internalType": "address[]", "name": "account", "type": "address[]" },
        { "internalType": "uinit256[]", "name": "token_id", "type": "uinit256[]" }
    ],
    name=balanceOfBatch,
    outputs=[{ "internalType": "uint256[]", "name": "", "type": "uint256[]" }],
    payable=false,
    stateMutability=view
)]
pub fn balance_of_batch(contract: &Contract) {
    // TODO: handle the offset bigger than usize
    let mut buf: [u8; 4] = contract.input_data[32..36].try_into().unwrap();
    let addr_offset = usize::from_be_bytes(buf) + 4;
    buf = contract.input_data[64..68].try_into().unwrap();
    let token_offset = usize::from_be_bytes(buf) + 4;

    let mut address_list = Vec::<Address>::new();
    let mut i = 0;

    buf = contract.input_data[addr_offset + 28..addr_offset + 32]
        .try_into()
        .unwrap();
    while i < usize::from_be_bytes(buf) {
        let byte20: [u8; 20] = contract.input_data
            [addr_offset + 44 + i * 32..addr_offset + 64 + i * 32]
            .try_into()
            .unwrap();
        let address = Address::from(byte20);
        address_list.push(address);
        i = i + 1;
    }

    let mut token_balance_list = Vec::<[u8; 32]>::new();
    i = 0;
    buf = contract.input_data[token_offset + 28..token_offset + 32]
        .try_into()
        .unwrap();
    while i < usize::from_be_bytes(buf) {
        let bytes32: [u8; 32] = contract.input_data
            [token_offset + 32 + i * 32..token_offset + 64 + i * 32]
            .try_into()
            .unwrap();
        let balance = get_token_balance(&address_list[i], &bytes32);
        token_balance_list.push(balance.bytes);
        i = i + 1;
    }
    ewasm_return_vec(&token_balance_list);
}

#[cfg(target_arch = "wasm32")]
fn do_transfer_from(from: &Address, to: &Address, token_id: &[u8; 32], value: Uint256) {
    let sender_storage_value = {
        let balance = get_token_balance(from, token_id);
        let origin_value = Uint256::from_be_bytes(balance.bytes);

        if origin_value < value {
            ewasm_api::revert();
        }

        let new_value = origin_value - value;
        let buffer = new_value.to_be_bytes();
        copy_into_storage_value(&buffer)
    };

    let recipient_storage_value = {
        let balance = get_token_balance(to, token_id);
        let origin_value = Uint256::from_be_bytes(balance.bytes);
        let new_value = origin_value + value;

        if origin_value > new_value {
            ewasm_api::revert();
        }

        let buffer = new_value.to_be_bytes();
        copy_into_storage_value(&buffer)
    };

    set_token_balance(from, token_id, &sender_storage_value);
    set_token_balance(to, token_id, &recipient_storage_value);
}

/// Implement ERC-1155 safeTransferFrom(address,address,uint256,uint256,bytes)
#[ewasm_lib_fn(f242432a,
    constant=true,
    inputs=[
        { "internalType": "address", "name": "from", "type": "address" },
        { "internalType": "address", "name": "to", "type": "address" },
        { "internalType": "uinit256", "name": "token_id", "type": "uinit256" },
        { "internalType": "uinit256", "name": "value", "type": "uinit256" },
        { "internalType": "bytes", "name": "data", "type": "bytes" }
    ],
    name=safeTransferFrom,
    outputs=[],
    payable=false,
    stateMutability=view
)]
pub fn safe_transfer_from(contract: &Contract) {
    let sender = ewasm_api::caller();
    let from = copy_into_address(&contract.input_data[16..36]);
    let to = copy_into_address(&contract.input_data[48..68]);
    let token_id: [u8; 32] = contract.input_data[68..100]
        .try_into()
        .expect("token id should be byte32");
    if !get_approval(&from, &sender) {
        ewasm_api::revert();
    }
    let value = {
        let value_data: [u8; 32] = contract.input_data[100..132].try_into().unwrap();
        Uint256::from_be_bytes(value_data)
    };

    do_transfer_from(&from, &to, &token_id, value);

    let topic: [u8; 32] =
        decode("c3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62")
            .unwrap()
            .try_into()
            .unwrap();
    log4(
        &Vec::<u8>::with_capacity(0), //TODO handler the byte
        &topic.into(),
        &Raw::from(sender).to_bytes32().into(),
        &Raw::from(from).to_bytes32().into(),
        &Raw::from(to).to_bytes32().into(),
    );
}

/// Implement ERC-1155 safeBatchTransferFrom(address,address,uint256[],uint256[],bytes)
#[ewasm_lib_fn("2eb2c2d6",
    constant=true,
    inputs=[
        { "internalType": "address", "name": "from", "type": "address" },
        { "internalType": "address", "name": "to", "type": "address" },
        { "internalType": "uinit256[]", "name": "token_id", "type": "uinit256[]" },
        { "internalType": "uinit256[]", "name": "value", "type": "uinit256[]" },
        { "internalType": "bytes", "name": "data", "type": "bytes" }
    ],
    name=safeBatchTransferFrom,
    outputs=[],
    payable=false,
    stateMutability=view
)]
pub fn safe_batch_transfer_from(contract: &Contract) {
    let sender = ewasm_api::caller();
    let from = copy_into_address(&contract.input_data[16..36]);
    let to = copy_into_address(&contract.input_data[48..68]);

    if !get_approval(&from, &sender) {
        ewasm_api::revert();
    }

    // TODO: handle the offset bigger than usize
    let mut buf: [u8; 4] = contract.input_data[96..100].try_into().unwrap();
    let token_offset = usize::from_be_bytes(buf) + 4;

    buf = contract.input_data[128..132].try_into().unwrap();
    let value_offset = usize::from_be_bytes(buf) + 4;

    buf = contract.input_data[token_offset + 28..token_offset + 32]
        .try_into()
        .unwrap();
    let token_length = usize::from_be_bytes(buf);
    let mut token_list = Vec::<[u8; 32]>::new();
    let mut i = 0;

    let mut output = Vec::<[u8; 32]>::new();
    while i < token_length {
        let bytes32: [u8; 32] = contract.input_data
            [token_offset + 32 + i * 32..token_offset + 64 + i * 32]
            .try_into()
            .unwrap();
        token_list.push(bytes32);
        i = i + 1;
    }

    i = 0;
    buf = contract.input_data[value_offset + 28..value_offset + 32]
        .try_into()
        .unwrap();
    while i < usize::from_be_bytes(buf) {
        let value = {
            let value_data: [u8; 32] = contract.input_data
                [value_offset + 32 + i * 32..value_offset + 64 + i * 32]
                .try_into()
                .unwrap();
            Uint256::from_be_bytes(value_data)
        };
        do_transfer_from(&from, &to, &token_list[i], value);
        i = i + 1;
    }

    let topic: [u8; 32] =
        decode("4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb")
            .unwrap()
            .try_into()
            .unwrap();
    log4(
        &Vec::<u8>::with_capacity(0), //TODO handler the byte
        &topic.into(),
        &Raw::from(ewasm_api::caller()).to_bytes32().into(),
        &Raw::from(from).to_bytes32().into(),
        &Raw::from(to).to_bytes32().into(),
    );
}
// URI(string,uint256): 6bb7ff708619ba0610cba295a58592e0451dee2622938c8755667688daf3529b

#[cfg(target_arch = "wasm32")]
pub fn mint(addr: &str, tokens: Vec<(&str, usize)>) {
    let address = {
        let byte20: [u8; 20] = decode(addr)
            .expect("address should be hex format")
            .try_into()
            .expect("address should be byte20");
        Address::from(byte20)
    };

    let topic: [u8; 32] =
        decode("c3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62")
            .unwrap()
            .try_into()
            .unwrap();
    for (token, value) in tokens.iter() {
        let token_id: [u8; 32] = decode(token)
            .expect("token id should be hex format")
            .try_into()
            .expect("token id should be byte32");
        set_token_balance(&address, &token_id, &Raw::from(*value).to_bytes32().into());
        log4(
            &Vec::<u8>::with_capacity(0), //TODO handler the byte
            &topic.into(),
            &Raw::from(0u32).to_bytes32().into(),
            &Raw::from(0u32).to_bytes32().into(),
            &Raw::from(address).to_bytes32().into(),
        );
    }
}
