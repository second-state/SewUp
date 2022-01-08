use crate::*;
use hex_literal::*;

#[test]
fn test_function_signature() {
    let mut sig: [u8; 4] = hex!("c48d6d5e");
    assert_eq!(get_function_signature("sendMessage(string,address)"), sig);
    sig = hex!("70a08231");
    assert_eq!(get_function_signature("balanceOf(address)"), sig);
}

#[test]
fn test_parse_fn_attr() {
    // without attr
    assert_eq!(parse_fn_attr("".to_string()), (None, "{}".to_string()));

    // with hex attr only
    assert_eq!(
        parse_fn_attr("a9059cbb".to_string()),
        (Some("a9059cbb".to_string()), "{}".to_string())
    );

    // with hex attr only
    assert_eq!(
        parse_fn_attr("a9059cbb, ".to_string()),
        (Some("a9059cbb".to_string()), "{}".to_string())
    );

    // with hex attr and full abijson
    // assert_eq!(parse_fn_attr(r#"
    //   a9059cbb,
    //   constant=false,
    //   inputs=[
    //         { "internalType": "address", "name": "recipient", "type": "address" },
    //         { "internalType": "uint256", "name": "amount", "type": "uint256" }
    //   ],
    //   name=transfer,
    //   outputs=[
    //         { "internalType": "bool", "name": "", "type": "bool" }
    //   ],
    //   payable=false,
    //   stateMutability=nonpayable,
    //   type=function
    // "#.to_string()), (Some("a9059cbb".to_string()), "{}".to_string()));
}
