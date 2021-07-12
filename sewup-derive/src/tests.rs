use crate::*;
use hex_literal::*;
#[test]
fn test_function_signature() {
    let mut sig: [u8; 4] = hex!("c48d6d5e");
    assert_eq!(get_function_signature("sendMessage(string,address)"), sig);
    sig = hex!("70a08231");
    assert_eq!(get_function_signature("balanceOf(address)"), sig);
}
