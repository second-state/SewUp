use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {
    sewup::token::erc721::mint(
        "8663DBF0cC68AaF37fC8BA262F2df4c666a41993",
        vec![
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000002",
        ],
    );
}

#[ewasm_main]
fn main() -> anyhow::Result<()> {
    let contract = sewup::primitives::Contract::new()?;
    match contract.get_function_selector()? {
        sewup::token::erc721::BALANCE_OF_SIG => sewup::token::erc721::balance_of(&contract),
        sewup::token::erc721::OWNER_OF_SIG => sewup::token::erc721::owner_of(&contract),
        _ => (),
    };
    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use hex_literal::hex;
    use sewup::erc721::{BALANCE_OF_SIG, OWNER_OF_SIG};
    use sewup_derive::ewasm_assert_eq;

    #[ewasm_test]
    fn test_execute_basic_operations() {
        let address_input = hex!("8663DBF0cC68AaF37fC8BA262F2df4c666a41993");
        let mut input_data = vec![0u8, 0u8, 0u8, 0u8];
        input_data.append(&mut address_input.to_vec());
        ewasm_assert_eq!(
            balance_of(input_data),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 2
            ]
        );

        let token1 = hex!("0000000000000000000000000000000000000000000000000000000000000001");
        let token2 = hex!("0000000000000000000000000000000000000000000000000000000000000002");
        let token3 = hex!("0000000000000000000000000000000000000000000000000000000000000003");

        ewasm_assert_eq!(
            owner_of(token1),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 134, 99, 219, 240, 204, 104, 170, 243, 127,
                200, 186, 38, 47, 45, 244, 198, 102, 164, 25, 147
            ]
        );
        ewasm_assert_eq!(
            owner_of(token2),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 134, 99, 219, 240, 204, 104, 170, 243, 127,
                200, 186, 38, 47, 45, 244, 198, 102, 164, 25, 147
            ]
        );

        ewasm_assert_eq!(
            owner_of(token3),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );
    }
}
