#[macro_export]
macro_rules! deploy_wasm {
    () => {
        "./target/wasm32-unknown-unknown/release/{}.deploy.wasm"
    };
}

#[macro_export]
macro_rules! deploy_file {
    () => {
        "./target/wasm32-unknown-unknown/release/{}.deploy"
    };
}

pub const DEFAULT_VALUE_LOG: usize = 17;
pub const DEFAULT_GAS: usize = 5;
pub const DEFAULT_GAS_PRICE: usize = 3_000_000;
