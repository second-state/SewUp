use std::convert::TryInto;

use crate::primitives::Contract;
use sewup_derive::ewasm_lib_fn;

#[cfg(target_arch = "wasm32")]
use super::helpers::{copy_into_address, get_token_balance};

#[cfg(target_arch = "wasm32")]
use crate::utils::ewasm_return_vec;

#[cfg(target_arch = "wasm32")]
use ewasm_api::{log3, log4, types::Address};

pub use super::erc721::{
    is_approved_for_all, set_approval_for_all, IS_APPROVED_FOR_ALL_SIG, SET_APPROVAL_FOR_ALL_SIG,
};

/// Implement ERC-1155 balanceOf(address,uint256)
/// ```json
/// {
///     "constant": true,
///     "inputs": [
///         { "internalType": "address", "name": "account", "type": "address" },
///         { "internalType": "uinit256", "name": "token_id", "type": "uinit256" }
///     ],
///     "name": "balanceOf",
///     "outputs": [{ "internalType": "uint256", "name": "", "type": "uint256" }],
///     "payable": false,
///     "stateMutability": "view",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn(00fdd58e)]
pub fn balance_of(contract: &Contract) {
    let address = copy_into_address(&contract.input_data[16..36]);
    let token_id: [u8; 32] = contract.input_data[36..68]
        .try_into()
        .expect("token id should be byte32");
    let balance = get_token_balance(&address, &token_id);
    ewasm_api::finish_data(&balance.bytes);
}

/// Implement ERC-1155 balanceOfBatch(address[],uint256[])
/// ```json
/// {
///     "constant": true,
///     "inputs": [
///         { "internalType": "address[]", "name": "account", "type": "address[]" },
///         { "internalType": "uinit256[]", "name": "token_id", "type": "uinit256[]" }
///     ],
///     "name": "balanceOfBatch",
///     "outputs": [{ "internalType": "uint256[]", "name": "", "type": "uint256[]" }],
///     "payable": false,
///     "stateMutability": "view",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn(4e1273f4)]
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

// safeBatchTransferFrom(address,address,uint256[],uint256[],bytes): 2eb2c2d6
// safeTransferFrom(address,address,uint256,uint256,bytes): f242432a
// TransferSingle(address,address,address,uint256,uint256): c3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62
// TransferBatch(address,address,address,uint256[],uint256[]): 4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb
// URI(string,uint256): 6bb7ff708619ba0610cba295a58592e0451dee2622938c8755667688daf3529b
