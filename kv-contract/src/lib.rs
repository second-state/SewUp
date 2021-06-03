use anyhow::Result;
use sewup::kv::{Feature, Store};
use sewup::primitives::Contract;
use sewup_derive::{ewasm_fn, ewasm_main, fn_sig};

mod errors;
use errors::KVError;

#[ewasm_fn]
fn empty_commit() -> Result<()> {
    let storage = Store::new()?;
    storage.commit()?;
    Ok(())
}

#[ewasm_fn]
fn check_version_and_features(version: u8, features: Vec<Feature>) -> Result<()> {
    let storage = Store::load(None)?;
    if storage.version != version {
        return Err(KVError::UnexpectVersion(storage.version).into());
    };
    let current_features = storage.features();
    if current_features != features {
        return Err(KVError::IncompatibleFeatures(current_features).into());
    };

    Ok(())
}

#[ewasm_main]
fn main() -> Result<()> {
    let contract = Contract::new()?;

    match contract.get_function_selector()? {
        fn_sig!(empty_commit) => empty_commit()?,
        fn_sig!(check_version_and_features) => {
            check_version_and_features(1, vec![Feature::Default])?
        }
        _ => return Err(KVError::UnknownHandle.into()),
    };

    Ok(())
}
