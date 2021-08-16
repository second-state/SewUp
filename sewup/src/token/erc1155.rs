use std::convert::TryInto;

use crate::primitives::Contract;
use sewup_derive::ewasm_lib_fn;

#[cfg(target_arch = "wasm32")]
use super::helpers::{copy_into_address, get_token_balance};

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

// isApprovedForAll(address,address): e985e9c5
// setApprovalForAll(address,bool): a22cb465
// balanceOfBatch(address,uint256): 830e3ae4
// safeBatchTransferFrom(address,address,uint256[],uint256[],bytes): 2eb2c2d6
// safeTransferFrom(address,address,uint256,uint256,bytes): f242432a
// TransferSingle(address,address,address,uint256,uint256): c3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62
// TransferBatch(address,address,address,uint256[],uint256[]): 4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb
// ApprovalForAll(address,address,bool): 17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31
// URI(string,uint256): 6bb7ff708619ba0610cba295a58592e0451dee2622938c8755667688daf3529b
