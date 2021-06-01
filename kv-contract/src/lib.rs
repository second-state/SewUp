use anyhow::Result;
use sewup::kv::Store;
use sewup::primitives::Contract;
use sewup_derive::{ewasm_fn, ewasm_main};

#[ewasm_fn]
fn new_bucket(contract: &Contract) {}

#[ewasm_main]
fn main() -> Result<()> {
    let contract = Contract::new()?;

    let storage = Store::new();

    match contract.get_function_selector()? {
        _ => (),
    };

    Ok(())
}
