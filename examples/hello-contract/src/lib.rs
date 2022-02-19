use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {}

#[ewasm_fn]
fn hello() -> Result<String, ()> {
    Ok("hello world".to_string())
}

#[ewasm_fn]
fn make_err() -> Result<String, ()> {
    Err(())
}

#[ewasm_main(auto, default = "Some error happen")]
fn main() -> Result<String, ()> {
    let contract = sewup::primitives::Contract::new().expect("contract should work");
    match contract
        .get_function_selector()
        .expect("function selector should work")
    {
        sewup_derive::ewasm_fn_sig!(hello) => hello(),
        sewup_derive::ewasm_fn_sig!(make_err) => make_err(),
        _ => Err(()),
    }
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_auto_assert_eq, ewasm_fn_sig, ewasm_output_from};

    #[ewasm_test]
    fn test_get_greeting() {
        ewasm_assert_eq!(
            hello(),
            vec![11, 0, 0, 0, 0, 0, 0, 0, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]
        );
        ewasm_assert_eq!(hello(), ewasm_output_from!("hello world".to_string()));
        ewasm_auto_assert_eq!(hello(), "hello world".to_string());
        ewasm_assert_eq!(make_err(), "Some error happen".as_bytes().to_vec());
    }
}
