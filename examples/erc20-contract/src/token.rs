use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {
    sewup::token::erc20::mint("8663DBF0cC68AaF37fC8BA262F2df4c666a41993", 1000);
}

#[ewasm_main]
fn main() -> anyhow::Result<()> {
    let contract = sewup::primitives::Contract::new()?;
    match contract.get_function_selector()? {
        sewup::token::erc20::BALANCE_OF_SIG => sewup::token::erc20::balance_of(&contract),
        sewup::token::erc20::TRANSFER_SIG => sewup::token::erc20::transfer(&contract),
        sewup::token::erc20::NAME_SIG => sewup::token::erc20::name("Demo"),
        sewup::token::erc20::SYMBOL_SIG => sewup::token::erc20::symbol("ETD"),
        sewup::token::erc20::DECIMALS_SIG => sewup::token::erc20::decimals(8),
        sewup::token::erc20::TOTAL_SUPPLY_SIG => sewup::token::erc20::total_supply(1000),
        sewup::token::erc20::APPROVE_SIG => sewup::token::erc20::approve(&contract),
        sewup::token::erc20::ALLOWANCE_SIG => sewup::token::erc20::allowance(&contract),
        sewup::token::erc20::TRANSFER_FROM_SIG => sewup::token::erc20::transfer_from(&contract),
        _ => (),
    };
    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use hex_literal::hex;
    use sewup::erc20::{
        BALANCE_OF_SIG, DECIMALS_SIG, NAME_SIG, SYMBOL_SIG, TOTAL_SUPPLY_SIG, TRANSFER_SIG,
    };
    use sewup_derive::ewasm_assert_eq;

    #[ewasm_test]
    fn test_execute_basic_operations() {
        ewasm_assert_eq!(
            name(),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 4, 68, 101, 109, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        ewasm_assert_eq!(
            symbol(),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 3, 69, 84, 68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        ewasm_assert_eq!(
            decimals(),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 8
            ]
        );
        ewasm_assert_eq!(
            total_supply(),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 3, 232
            ]
        );

        let balance_input = hex!("8663DBF0cC68AaF37fC8BA262F2df4c666a41993");
        let mut input_data = vec![0u8, 0u8, 0u8, 0u8];
        input_data.append(&mut balance_input.to_vec());
        ewasm_assert_eq!(
            balance_of(input_data),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 3, 232
            ]
        );

        let transfer_recipent = hex!("0000000000000000000000000000000000000001");
        let transfer_value =
            hex!("0000000000000000000000000000000000000000000000000000000000000009");
        input_data = vec![0u8, 0u8, 0u8, 0u8];
        input_data.append(&mut transfer_recipent.to_vec());
        input_data.append(&mut transfer_value.to_vec());

        // assert the transfer() function of contract and call by "8663DBF0cC68AaF37fC8BA262F2df4c666a41993"
        ewasm_assert_eq!(
            transfer(input_data) by "8663DBF0cC68AaF37fC8BA262F2df4c666a41993",
            vec![]
        );

        let balance_input = hex!("8663DBF0cC68AaF37fC8BA262F2df4c666a41993");
        let mut input_data = vec![0u8, 0u8, 0u8, 0u8];
        input_data.append(&mut balance_input.to_vec());
        ewasm_assert_eq!(
            balance_of(input_data),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 3, 223
            ]
        );
    }
}
