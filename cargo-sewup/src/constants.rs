#[macro_export]
macro_rules! deploy_wasm {
    () => {
        "./target/wasm32-unknown-unknown/release/{}.deploy.wasm"
    };
}

pub const DEFAULT_GAS: usize = 500_000_000;
pub const DEFAULT_GAS_PRICE: usize = 1;
