use sewup_derive::{ewasm_constructor, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {
    sewup::token::erc721::mint(
        "8663DBF0cC68AaF37fC8BA262F2df4c666a41993",
        vec![
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
        ],
    );
}

#[ewasm_main]
fn main() -> anyhow::Result<()> {
    let contract = sewup::primitives::Contract::new()?;
    match contract.get_function_selector()? {
        sewup::token::erc721::BALANCE_OF_SIG => sewup::token::erc721::balance_of(&contract),
        sewup::token::erc721::OWNER_OF_SIG => sewup::token::erc721::owner_of(&contract),
        sewup::token::erc721::TRANSFER_SIG => sewup::token::erc721::transfer(&contract),
        sewup::token::erc721::TRANSFER_FROM_SIG => sewup::token::erc721::transfer_from(&contract),
        sewup::token::erc721::APPROVE_SIG => sewup::token::erc721::approve(&contract),
        sewup::token::erc721::GET_APPROVED_SIG => sewup::token::erc721::get_approved(&contract),
        sewup::token::erc721::SET_APPROVAL_FOR_ALL_SIG => {
            sewup::token::erc721::set_approval_for_all(&contract)
        }
        sewup::token::erc721::IS_APPROVED_FOR_ALL_SIG => {
            sewup::token::erc721::is_approved_for_all(&contract)
        }
        _ => (),
    };
    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use hex_literal::hex;
    use sewup::erc721::{BALANCE_OF_SIG, OWNER_OF_SIG, TRANSFER_SIG};
    use sewup_derive::ewasm_assert_eq;

    #[ewasm_test]
    fn test_execute_basic_operations() {
        let address_input = hex!("8663DBF0cC68AaF37fC8BA262F2df4c666a41993");
        let mut input_data = vec![0u8, 0u8, 0u8, 0u8];
        input_data.extend_from_slice(&address_input);
        ewasm_assert_eq!(
            balance_of(input_data),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 3
            ]
        );

        let token1 = hex!("0000000000000000000000000000000000000000000000000000000000000001");
        let token2 = hex!("0000000000000000000000000000000000000000000000000000000000000002");
        let token4 = hex!("0000000000000000000000000000000000000000000000000000000000000004");

        ewasm_assert_eq!(
            owner_of(token1),
            hex!("0000000000000000000000008663DBF0cC68AaF37fC8BA262F2df4c666a41993").to_vec()
        );
        ewasm_assert_eq!(
            owner_of(token2),
            hex!("0000000000000000000000008663DBF0cC68AaF37fC8BA262F2df4c666a41993").to_vec()
        );

        ewasm_assert_eq!(
            owner_of(token4),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        let transfer_recipent = hex!("0000000000000000000000000000000000000001");
        input_data = vec![0u8, 0u8, 0u8, 0u8];
        input_data.append(&mut transfer_recipent.to_vec());
        input_data.append(&mut token1.to_vec());

        ewasm_assert_eq!(
            transfer(input_data) by "8663DBF0cC68AaF37fC8BA262F2df4c666a41993",
            vec![]
        );
        ewasm_assert_eq!(
            owner_of(token1),
            hex!("0000000000000000000000000000000000000000000000000000000000000001").to_vec()
        );
    }
}
