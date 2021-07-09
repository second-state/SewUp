use serde_derive::{Deserialize, Serialize};

use sewup::kv::Feature;
use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test, Value};

mod errors;
use errors::KVError;

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Value)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[ewasm_fn]
fn empty_commit() -> anyhow::Result<()> {
    let storage = sewup::kv::Store::new()?;
    let size = storage.commit()?;
    if size != 8u32 {
        return Err(KVError::UnexpectedDBSize(size).into());
    }
    Ok(())
}

#[ewasm_fn]
fn check_version_and_features(version: u8, features: Vec<Feature>) -> anyhow::Result<()> {
    let storage = sewup::kv::Store::load(None)?;
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
fn check_empty_storage_size(size: u32) -> anyhow::Result<()> {
    let storage = sewup::kv::Store::load(None)?;
    let load_size = storage.load_size();
    if load_size != size {
        return Err(KVError::UnexpectedDBSize(load_size).into());
    }
    Ok(())
}

#[ewasm_fn]
fn add_buckets() -> anyhow::Result<()> {
    use sewup::types::{Raw, Row};

    let mut storage = sewup::kv::Store::load(None)?;
    let bucket1 = storage.bucket::<Raw, Raw>("bucket1")?;
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
fn check_buckets(buckets: Vec<String>) -> anyhow::Result<()> {
    let mut storage = sewup::kv::Store::load(None)?;
    let mut current_buckets = storage.buckets();
    current_buckets.sort();
    if current_buckets != buckets {
        return Err(KVError::IncorrectBuckets(current_buckets).into());
    }
    Ok(())
}

#[ewasm_fn]
fn drop_bucket_than_check(name: &str, remine_buckets: Vec<String>) -> anyhow::Result<()> {
    let mut storage = sewup::kv::Store::load(None)?;
    storage.drop_bucket(name)?;
    storage.commit()?;

    let s = sewup::kv::Store::load(None)?;
    let mut current_buckets = s.buckets();
    current_buckets.sort();
    if current_buckets != remine_buckets {
        return Err(KVError::IncorrectBuckets(current_buckets).into());
    }
    Ok(())
}

#[ewasm_fn]
fn new_bucket_with_specific_struct() -> anyhow::Result<()> {
    use sewup::types::{Raw, Row};

    let mut storage = sewup::kv::Store::new()?;
    let mut bucket1 = storage.bucket::<Raw, Row>("bucket1")?;
    let mut bucket2 = storage.bucket::<Raw, SimpleStruct>("bucket2")?;

    bucket1.set(
        b"jovy".into(),
        "A faith keep me up and away from fall".to_string().into(),
    )?;
    let simple_struct = SimpleStruct {
        trust: true,
        description: "An action without doubt".to_string(),
    };
    bucket2.set(b"ant".into(), simple_struct)?;

    storage.save(bucket1);
    storage.save(bucket2);

    storage.commit()?;
    Ok(())
}

#[ewasm_fn]
fn check_objects_in_bucket() -> anyhow::Result<()> {
    use sewup::types::{Raw, Row};

    let mut storage = sewup::kv::Store::load(None)?;
    let mut bucket1 = storage.bucket::<Raw, Row>("bucket1")?;
    let mut bucket2 = storage.bucket::<Raw, SimpleStruct>("bucket2")?;

    if let Some(faith) = bucket1.get(b"jovy".into())? {
        if faith.to_utf8_string()? !=
            "A faith keep me up and away from fall\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}".to_string() {
            return Err(KVError::ValueError(faith.to_utf8_string()? ).into());
        }
    } else {
        return Err(KVError::ValueNotFound.into());
    }

    if let Some(simple_struct) = bucket2.get(b"ant".into())? {
        if !simple_struct.trust {
            return Err(KVError::ValueError("struct trust not true".to_string()).into());
        }
        if simple_struct.description != "An action without doubt".to_string() {
            return Err(KVError::ValueError(simple_struct.description).into());
        }
    } else {
        return Err(KVError::ValueNotFound.into());
    }
    bucket2.set(b"bug".into(), SimpleStruct::default())?;

    storage.save(bucket1);
    storage.save(bucket2);

    storage.commit()?;

    Ok(())
}

#[ewasm_fn]
fn delete_object_in_bucket() -> anyhow::Result<()> {
    use sewup::types::{Raw, Row};

    let mut storage = sewup::kv::Store::load(None)?;
    let mut bucket2 = storage.bucket::<Raw, SimpleStruct>("bucket2")?;

    if bucket2.get(b"bug".into())?.is_none() {
        return Err(KVError::ValueError("there should be a bug for testing".to_string()).into());
    }

    bucket2.remove(b"bug".into())?;

    if bucket2.get(b"bug".into())?.is_some() {
        return Err(
            KVError::ValueError("there should be no bug after deleting".to_string()).into(),
        );
    }

    Ok(())
}

#[ewasm_fn]
fn non_regist_function() -> anyhow::Result<()> {
    // A function forget to regist
    Ok(())
}

#[ewasm_main]
fn main() -> anyhow::Result<()> {
    let contract = sewup::primitives::Contract::new()?;

    match contract.get_function_selector()? {
        ewasm_fn_sig!(empty_commit) => empty_commit()?,
        ewasm_fn_sig!(check_version_and_features) => {
            check_version_and_features(0, vec![Feature::Default])?
        }
        ewasm_fn_sig!(check_empty_storage_size) => check_empty_storage_size(8u32)?,
        ewasm_fn_sig!(add_buckets) => add_buckets()?,
        ewasm_fn_sig!(check_buckets) => {
            check_buckets(vec!["bucket1".to_string(), "bucket2".to_string()])?
        }
        ewasm_fn_sig!(drop_bucket_than_check) => {
            drop_bucket_than_check("bucket1", vec!["bucket2".to_string()])?
        }

        // Following handler is for other test
        ewasm_fn_sig!(new_bucket_with_specific_struct) => new_bucket_with_specific_struct()?,
        ewasm_fn_sig!(check_objects_in_bucket) => check_objects_in_bucket()?,
        ewasm_fn_sig!(delete_object_in_bucket) => delete_object_in_bucket()?,
        _ => return Err(KVError::UnknownHandle.into()),
    };

    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_err_output};

    #[ewasm_test]
    fn test_execute_storage_operations() {
        ewasm_assert_ok!(empty_commit());

        ewasm_assert_eq!(
            non_regist_function(),
            ewasm_err_output!(KVError::UnknownHandle)
        );

        ewasm_assert_ok!(check_version_and_features());

        ewasm_assert_ok!(check_empty_storage_size());

        ewasm_assert_ok!(add_buckets());

        ewasm_assert_ok!(check_buckets());

        ewasm_assert_ok!(drop_bucket_than_check());
    }

    #[ewasm_test]
    fn test_execute_bucket_operations() {
        ewasm_assert_ok!(new_bucket_with_specific_struct());
        ewasm_assert_ok!(check_objects_in_bucket());
        ewasm_assert_ok!(delete_object_in_bucket());
    }
}
