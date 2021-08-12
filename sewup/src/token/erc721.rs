use std::convert::TryInto;

use crate::primitives::Contract;
use crate::types::Raw;
use sewup_derive::ewasm_lib_fn;

pub use super::erc20::{balance_of, name, symbol, BALANCE_OF_SIG, NAME_SIG, SYMBOL_SIG};

#[cfg(target_arch = "wasm32")]
use super::helpers::{get_token_owner, set_balance, set_token_owner};

#[cfg(target_arch = "wasm32")]
use ewasm_api::types::Address;

#[cfg(target_arch = "wasm32")]
use hex::decode;

#[cfg(not(target_arch = "wasm32"))]
use super::helpers::Address;

/// Implement ERC-721 owner_of()
/// ```json
/// {
///     "constant": true,
///     "inputs": [{ "name": "_tokenId", "type": "uint256" }],
///     "name": "ownerOf",
///     "outputs": [{ "name": "_owner", "type": "address" }],
///     "payable": false,
///     "stateMutability": "view",
///     "type": "function"
/// }
/// ```
#[ewasm_lib_fn("6352211e")]
pub fn owner_of(contract: &Contract) {
    let token_id: [u8; 32] = contract.input_data[4..36]
        .try_into()
        .expect("token id should be byte32");
    let owner = get_token_owner(&token_id);
    ewasm_api::finish_data(&Raw::from(owner).as_bytes().to_vec());
}

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
