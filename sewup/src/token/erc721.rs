#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

use crate::primitives::Contract;
#[cfg(target_arch = "wasm32")]
use crate::types::Raw;
use sewup_derive::ewasm_lib_fn;

pub use super::erc20::{
    balance_of, name, symbol, BALANCE_OF_ABI, BALANCE_OF_SIG, NAME_ABI, NAME_SIG, SYMBOL_ABI,
    SYMBOL_SIG,
};

#[cfg(target_arch = "wasm32")]
use super::helpers::{
    copy_into_address, copy_into_storage_value, get_approval, get_balance, get_token_approval,
    get_token_owner, set_approval, set_balance, set_token_approval, set_token_owner,
};

#[cfg(target_arch = "wasm32")]
use crate::utils::ewasm_return_bool;

#[cfg(target_arch = "wasm32")]
use bitcoin::util::uint::Uint256;
#[cfg(target_arch = "wasm32")]
use hex::decode;

#[cfg(target_arch = "wasm32")]
use ewasm_api::{log3, log4, types::Address};

/// Implement ERC-721 owner_of()
#[ewasm_lib_fn("6352211e",
    constant=true,
    inputs=[{ "name": "_tokenId", "type": "uint256" }],
    name=ownerOf,
    outputs=[{ "name": "_owner", "type": "address" }],
    payable=false,
    stateMutability=view
)]
pub fn owner_of(contract: &Contract) {
    let token_id: [u8; 32] = contract.input_data[4..36]
        .try_into()
        .expect("token id should be byte32");
    let owner = get_token_owner(&token_id);
    ewasm_api::finish_data(&Raw::from(owner).as_bytes().to_vec());
}

#[cfg(target_arch = "wasm32")]
fn do_transfer(owner: Address, to: Address, token_id: [u8; 32]) {
    let mut balance = get_balance(&owner);
    let mut value = Uint256::from_be_bytes(balance.bytes)
        - Uint256::from_u64(1u64).expect("uint256 one should valid");
    let mut buffer = value.to_be_bytes();
    set_balance(&owner, &copy_into_storage_value(&buffer));

    balance = get_balance(&to);
    value = Uint256::from_be_bytes(balance.bytes)
        + Uint256::from_u64(1u64).expect("uint256 one should valid");
    buffer = value.to_be_bytes();
    set_balance(&to, &copy_into_storage_value(&buffer));

    set_token_owner(&token_id, &to);
    set_token_approval(&token_id, &to);

    let topic: [u8; 32] =
        decode("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
            .unwrap()
            .try_into()
            .unwrap();
    log4(
        &Vec::<u8>::with_capacity(0),
        &topic.into(),
        &Raw::from(owner).to_bytes32().into(),
        &Raw::from(to).to_bytes32().into(),
        &token_id.into(),
    );
}

/// Implement ERC-721 transfer()
#[ewasm_lib_fn("a9059cbb",
  constant=false,
  inputs=[
    { "name": "_to", "type": "address" },
    { "name": "_tokenId", "type": "uint256" }
  ],
  name=transfer,
  outputs=[],
  payable=false,
  stateMutability=nonpayable
)]
pub fn transfer(contract: &Contract) {
    let to = copy_into_address(&contract.input_data[16..36]);
    let token_id: [u8; 32] = contract.input_data[36..68]
        .try_into()
        .expect("token id should be byte32");
    let sender = ewasm_api::caller();
    let owner = get_token_owner(&token_id);

    if owner != sender {
        ewasm_api::revert();
    }

    do_transfer(owner, to, token_id);
}

/// Implement ERC-721 transferFrom(address,address,uint256)
#[ewasm_lib_fn("23b872dd",
  constant=false,
  inputs=[
    { "name": "_from", "type": "address" },
    { "name": "_to", "type": "address" },
    { "name": "_tokenId", "type": "uint256" }
  ],
  name=transferFrom,
  outputs=[],
  payable=false,
  stateMutability=nonpayable
)]
pub fn transfer_from(contract: &Contract) {
    let sender = ewasm_api::caller();
    let owner = copy_into_address(&contract.input_data[16..36]);
    let to = copy_into_address(&contract.input_data[48..68]);
    let token_id = contract.input_data[68..100].try_into().unwrap();

    if sender != get_token_approval(&token_id) && !get_approval(&owner, &sender) {
        ewasm_api::revert();
    }

    do_transfer(owner, to, token_id);
}

/// Implement ERC-721 approve(address,uint256)
#[ewasm_lib_fn("095ea7b3",
    constant=false,
    inputs=[
        { "name": "_to", "type": "address" },
        { "name": "_tokenId", "type": "uint256" }
    ],
    name=approve,
    outputs=[],
    payable=false,
    stateMutability=nonpayable
)]
pub fn approve(contract: &Contract) {
    let sender = ewasm_api::caller();
    let spender = copy_into_address(&contract.input_data[16..36]);
    let token_id: [u8; 32] = contract.input_data[36..68]
        .try_into()
        .expect("token id should be byte32");
    set_token_approval(&token_id, &spender);
    let topic: [u8; 32] =
        decode("8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925")
            .unwrap()
            .try_into()
            .unwrap();
    log4(
        &Vec::<u8>::with_capacity(0),
        &topic.into(),
        &Raw::from(sender).to_bytes32().into(),
        &Raw::from(spender).to_bytes32().into(),
        &token_id.into(),
    );
}

/// Implement ERC-721 getApproved(uint256)
#[ewasm_lib_fn("081812fc",
    constant=false,
    inputs=[ { "name": "_tokenId", "type": "uint256" } ],
    name=getApproved,
    outputs=[{ "name": "_owner", "type": "address" }],
    payable=false,
    stateMutability=nonpayable
)]
pub fn get_approved(contract: &Contract) {
    let token_id: [u8; 32] = contract.input_data[4..36]
        .try_into()
        .expect("token id should be byte32");
    let spender = get_token_approval(&token_id);
    ewasm_api::finish_data(&Raw::from(spender).as_bytes().to_vec());
}

/// Implement ERC-721 setApprovalForAll(address,bool)
#[ewasm_lib_fn("a22cb465",
    constant=false,
    inputs=[
        { "name": "_operator", "type": "address" },
        { "name": "_approved", "type": "bool" }
    ],
    name=setApprovalForAll,
    outputs=[],
    payable=false,
    stateMutability=nonpayable
)]
pub fn set_approval_for_all(contract: &Contract) {
    let sender = ewasm_api::caller();
    let operator = copy_into_address(&contract.input_data[16..36]);
    let is_approved = contract.input_data[67] == 1;
    set_approval(&sender, &operator, is_approved);

    let topic: [u8; 32] =
        decode("17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31")
            .unwrap()
            .try_into()
            .unwrap();
    log3(
        &Vec::<u8>::with_capacity(0),
        &topic.into(),
        &Raw::from(sender).to_bytes32().into(),
        &Raw::from(operator).to_bytes32().into(),
    );
}

/// Implement ERC-721 isApprovedForAll(address,address)
#[ewasm_lib_fn("e985e9c5",
    constant=false,
    inputs=[
        { "name": "owner", "type": "address" },
        { "name": "operator", "type": "address" }
    ],
    name=isApprovedForAll,
    outputs=[{ "name": "_approved", "type": "bool" }],
    payable=false,
    stateMutability=nonpayable
)]
pub fn is_approved_for_all(contract: &Contract) {
    let owner = copy_into_address(&contract.input_data[16..36]);
    let operator = copy_into_address(&contract.input_data[48..68]);
    ewasm_return_bool(get_approval(&owner, &operator));
}

/// Implement ERC-721 tokenMetadata(uint256)
#[ewasm_lib_fn("6914db60",
    constant=true,
    inputs=[ { "name": "_tokenId", "type": "uint256" } ],
    name=tokenMetadata,
    outputs=[ { "name": "_infoUrl", "type": "string" } ],
    payable=false,
    stateMutability=view
)]
pub fn token_metadata(contract: &Contract) {
    // TODO
    // https://github.com/second-state/SewUp/issues/161
}

/// Implement ERC-721 safeTransferFrom(address,address,uint256,bytes)
/// @dev Throws unless `msg.sender` is the current owner, an authorized
/// operator, or the approved address for this NFT. Throws if `_from` is
/// not the current owner. Throws if `_to` is the zero address. Throws if
/// `_tokenId` is not a valid NFT. When transfer is complete, this function
/// checks if `_to` is a smart contract (code size > 0). If so, it calls
/// `onERC721Received` on `_to` and throws if the return value is not
/// `bytes4(keccak256("onERC721Received(address,address,uint256,bytes)"))`.
#[ewasm_lib_fn("b88d4fde")]
pub fn safe_transfer_from_with_data(contract: &Contract) {
    // TODO
    // https://github.com/second-state/SewUp/issues/160
}

/// Implement ERC-721 safeTransferFrom(address,address,uint256)
#[ewasm_lib_fn("42842e0e",
  constant=false,
  inputs=[
    { "name": "_from", "type": "address" },
    { "name": "_to", "type": "address" },
    { "name": "_tokenId", "type": "uint256" }
  ],
  name=safeTransferFrom,
  outputs=[],
  payable=false,
  stateMutability=nonpayable
)]
pub fn safe_transfer_from(contract: &Contract) {
    // TODO
    // https://github.com/second-state/SewUp/issues/160
}

#[cfg(target_arch = "wasm32")]
pub fn mint(addr: &str, tokens: Vec<&str>) {
    let address = {
        let byte20: [u8; 20] = decode(addr)
            .expect("address should be hex format")
            .try_into()
            .expect("address should be byte20");
        Address::from(byte20)
    };

    let topic: [u8; 32] =
        decode("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
            .unwrap()
            .try_into()
            .unwrap();
    for token in tokens.iter() {
        let token_id: [u8; 32] = decode(token)
            .expect("token id should be hex format")
            .try_into()
            .expect("token id should be byte32");
        set_token_owner(&token_id, &address);
        log4(
            &Vec::<u8>::with_capacity(0),
            &topic.into(),
            &Raw::from(0u32).to_bytes32().into(),
            &Raw::from(address).to_bytes32().into(),
            &token_id.into(),
        );
    }

    set_balance(&address, &Raw::from(tokens.len()).to_bytes32().into());
}
