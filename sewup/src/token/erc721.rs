use std::convert::TryInto;

use crate::types::Raw;

pub use super::erc20::{balance_of, BALANCE_OF_SIG};
#[cfg(target_arch = "wasm32")]
use super::helpers::{set_balance, set_token_owner};

#[cfg(target_arch = "wasm32")]
use ewasm_api::types::Address;

#[cfg(target_arch = "wasm32")]
use hex::decode;

#[cfg(not(target_arch = "wasm32"))]
use super::helpers::Address;

// ownerOf(uint256): 6352211e
// safeTransferFrom(address,address,uint256,bytes): b88d4fde
// safeTransferFrom(address,address,uint256): 42842e0e
// transferFrom(address,address,uint256): 23b872dd
// approve(address,uint256): 095ea7b3
// setApprovalForAll(address,bool): a22cb465
// getApproved(uint256): 081812fc
// isApprovedForAll(address,address): e985e9c5

#[cfg(target_arch = "wasm32")]
pub fn mint(addr: &str, tokens: Vec<&str>) {
    let byte20: [u8; 20] = decode(addr)
        .expect("address should be hex format")
        .try_into()
        .expect("address should be byte20");
    let address = Address::from(byte20);

    for token in tokens.iter() {
        let token_id: [u8; 32] = decode(token)
            .expect("token id should be hex format")
            .try_into()
            .expect("token id should be byte32");
        set_token_owner(&token_id, &address);
    }

    set_balance(&address, &Raw::from(tokens.len()).to_bytes32().into());
}
