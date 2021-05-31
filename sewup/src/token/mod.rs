#[allow(dead_code)]
#[allow(unused_variables)]
#[cfg(test)]
mod tests;

#[cfg(not(test))]
pub mod helpers;

// This will be test after compiled into wasm
#[cfg(not(test))]
pub mod erc20;
