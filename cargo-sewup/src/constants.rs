#[macro_export]
macro_rules! deploy_wasm {
    () => {
        "./target/wasm32-unknown-unknown/release/{}.deploy.wasm"
    };
}

pub const DEFAULT_GAS: usize = 500_000_000;
pub const DEFAULT_GAS_PRICE: usize = 1;

#[macro_export]
macro_rules! default_cargo_template {
    () => {
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2018"

[lib]
path = "src/lib.rs"
crate-type = ["cdylib"]

# See the following examples
# https://github.com/second-state/SewUp/tree/main/examples/hello-contract
# https://github.com/second-state/SewUp/tree/main/examples/default-contract

[dependencies]
sewup = "*"
sewup-derive = "*"
anyhow = "*"
# thiserror = "*"

[profile.release]
incremental = false
panic = "abort"
lto = true
opt-level = "z"

[profile.release.package.{}]
incremental = false
opt-level = "z"

[features]
constructor = []
constructor-test = []"#
    };
}

pub const DEFAULT_CONTRACT: &str = r#"
use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {}

#[ewasm_fn]
fn hello() -> anyhow::Result<String> {
    Ok("hello world".to_string())
}

#[ewasm_main]
fn main() -> anyhow::Result<()> {
    let contract = sewup::primitives::Contract::new()?;
    match contract.get_function_selector()? {
        ewasm_fn_sig!(hello) => hello()?,
        _ => panic!("unknown handle"),
    };
    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok};

    #[ewasm_test]
    fn test_get_greeting() {
        // The default mode does not use anything successful return from Rust
        // The only thing is return ok or not
        ewasm_assert_eq!(hello(), vec![]);
        ewasm_assert_ok!(hello());
    }
}"#;

#[macro_export]
macro_rules! rusty_cargo_template {
    () => {
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2018"

[lib]
path = "src/lib.rs"
crate-type = ["cdylib"]

# See the following examples
# https://github.com/second-state/SewUp/tree/main/examples/rusty-contract

[dependencies]
sewup = "*"
sewup-derive = "*"

[profile.release]
incremental = false
panic = "abort"
lto = true
opt-level = "z"

[profile.release.package.{}]
incremental = false
opt-level = "z"

[features]
constructor = []
constructor-test = []"#
    };
}

pub const RUSTY_CONTRACT: &str = r#"
use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {}

#[ewasm_fn]
fn handler() -> Result<(), &'static str> {
    Ok(())
}

#[ewasm_main(rusty)]
fn main() -> Result<(), &'static str> {
    use sewup::primitives::Contract;
    use sewup_derive::ewasm_input_from;

    let contract = Contract::new().map_err(|_| "NewContractError")?;
    match contract
        .get_function_selector()
        .map_err(|_| "FailGetFnSelector")?
    {
        ewasm_fn_sig!(handler) => handler()?,
        _ => return Err("UnknownHandle"),
    };

    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup::primitives::Contract;
    use sewup_derive::{ewasm_assert_eq, ewasm_rusty_assert_ok, ewasm_rusty_err_output};

    #[ewasm_test]
    fn test_handler_ok() {
        ewasm_rusty_assert_ok!(handler());
    }
}"#;

pub const AUTO_CONTRACT: &str = r#"
use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

#[ewasm_constructor]
fn constructor() {}

#[ewasm_fn]
fn handler() -> anyhow::Result<sewup::primitives::EwasmAny> {
    Ok(().into())
}


#[ewasm_main(auto)]
fn main() -> anyhow::Result<sewup::primitives::EwasmAny> {
    use sewup_derive::ewasm_input_from;
    let contract = sewup::primitives::Contract::new()?;

    match contract.get_function_selector()? {
        ewasm_fn_sig!(handler) => handler(),
        _ => return Err(anyhow::anyhow!("Unknow Error")),
    }
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_auto_assert_eq, ewasm_err_output};

    #[ewasm_test]
    fn test_handler() {
        ewasm_auto_assert_eq!(handler(), ());
    }
}"#;
