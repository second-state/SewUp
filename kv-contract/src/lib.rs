use anyhow::Result;
use sewup::kv::{Feature, Store};
use sewup::primitives::Contract;
use sewup::types::Row;
use sewup_derive::{ewasm_fn, ewasm_main, fn_sig};

mod errors;
use errors::KVError;

const EMPTY_DB_SIZE: u32 = 8;

#[ewasm_fn]
fn empty_commit() -> Result<()> {
    let storage = Store::new()?;
    let size = storage.commit()?;
    if size != EMPTY_DB_SIZE {
        return Err(KVError::UnexpectedDBSize(size).into());
    }
    Ok(())
}

#[ewasm_fn]
fn check_version_and_features(version: u8, features: Vec<Feature>) -> Result<()> {
    let storage = Store::load(None)?;
    if storage.version() != version {
        return Err(KVError::UnexpectVersion(storage.version()).into());
    };
    let current_features = storage.features();
    if current_features != features {
        return Err(KVError::IncompatibleFeatures(current_features).into());
    };

    Ok(())
}

#[ewasm_fn]
fn check_empty_storage_size(size: u32) -> Result<()> {
    let storage = Store::load(None)?;
    let load_size = storage.load_size();
    if load_size != size {
        return Err(KVError::UnexpectedDBSize(load_size).into());
    }
    Ok(())
}

#[ewasm_fn]
fn add_buckets() -> Result<()> {
    let mut storage = Store::load(None)?;
    let bucket1 = storage.bucket::<Row, Row>("bucket1")?;
    let bucket2 = storage.bucket::<Row, Row>("bucket2")?;
    if !bucket1.is_empty() {
        return Err(KVError::BucketError("inited bucket should be empty.".to_string()).into());
    }
    if bucket2.len() != 0 {
        return Err(
            KVError::BucketError(format!("bucket len {} incorrect.", bucket2.len())).into(),
        );
    }
    storage.save(bucket1);
    storage.save(bucket2);
    storage.commit()?;
    Ok(())
}

#[ewasm_fn]
fn check_buckets(buckets: Vec<String>) -> Result<()> {
    let mut storage = Store::load(None)?;
    let mut current_buckets = storage.buckets();
    current_buckets.sort();
    if current_buckets != buckets {
        return Err(KVError::IncorrectBuckets(current_buckets).into());
    }
    Ok(())
}

#[ewasm_fn]
fn drop_bucket_than_check(name: &str, remine_buckets: Vec<String>) -> Result<()> {
    let mut storage = Store::load(None)?;
    storage.drop_bucket(name)?;
    storage.commit()?;

    let s = Store::load(None)?;
    let mut current_buckets = s.buckets();
    current_buckets.sort();
    if current_buckets != remine_buckets {
        return Err(KVError::IncorrectBuckets(current_buckets).into());
    }
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
        fn_sig!(check_empty_storage_size) => check_empty_storage_size(EMPTY_DB_SIZE)?,
        fn_sig!(add_buckets) => add_buckets()?,
        fn_sig!(check_buckets) => {
            check_buckets(vec!["bucket1".to_string(), "bucket2".to_string()])?
        }
        fn_sig!(drop_bucket_than_check) => {
            drop_bucket_than_check("bucket1", vec!["bucket2".to_string()])?
        }
        _ => return Err(KVError::UnknownHandle.into()),
    };

    Ok(())
}
