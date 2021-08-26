use serde_derive::{Deserialize, Serialize};
use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test, Value};

mod errors;
use errors::KVError;

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Value)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[derive(Default, Serialize, Deserialize)]
struct InputPair(u32, Vec<u8>);

#[ewasm_constructor]
fn constructor() {
    use sewup::types::{Raw, Row};
    let mut storage =
        sewup::kv::Store::new().expect("there is no return for constructor currently");
    let bucket1 = storage
        .bucket::<Raw, Row>("bucket1")
        .expect("there is no return for constructor currently");
    let bucket2 = storage
        .bucket::<Raw, SimpleStruct>("bucket2")
        .expect("there is no return for constructor currently");
    storage.save(bucket1);
    storage.save(bucket2);
    storage
        .commit()
        .expect("there is no return for constructor currently");
}

#[ewasm_fn]
fn put_pair_to_bucket1(pair: InputPair) -> anyhow::Result<sewup::primitives::EwasmAny> {
    use sewup::types::{Raw, Row};
    let mut storage = sewup::kv::Store::load(None)?;
    let mut bucket1 = storage.bucket::<Raw, Row>("bucket1")?;
    bucket1.set(Raw::from(pair.0), Raw::from(pair.1).into())?;
    storage.save(bucket1);
    storage.commit()?;
    Ok(().into())
}

#[ewasm_fn]
fn get_value_to_bucket1(key: u32) -> anyhow::Result<sewup::primitives::EwasmAny> {
    use sewup::types::{Raw, Row};
    let mut storage =
        sewup::kv::Store::load(None).expect("there is no return for constructor currently");
    let bucket1 = storage.bucket::<Raw, Row>("bucket1")?;
    let value = bucket1.get(Raw::from(key))?.map(|x| x.into_u8_vec());
    Ok(sewup::primitives::EwasmAny::from(value))
}

#[ewasm_fn]
fn check_ver_and_feat(
    version: u8,
    features: Vec<sewup::kv::Feature>,
) -> anyhow::Result<sewup::primitives::EwasmAny> {
    let storage = sewup::kv::Store::load(None)?;
    if storage.version() != version {
        return Err(KVError::UnexpectedVersion(storage.version()).into());
    };
    let current_features = storage.features();
    if current_features != features {
        return Err(KVError::IncompatibleFeatures(current_features).into());
    };

    Ok(().into())
}

#[ewasm_fn]
fn check_buckets(buckets: Vec<String>) -> anyhow::Result<sewup::primitives::EwasmAny> {
    let mut storage = sewup::kv::Store::load(None)?;
    let mut current_buckets = storage.buckets();
    current_buckets.sort();

    if current_buckets != buckets {
        return Err(KVError::IncorrectBuckets(current_buckets).into());
    }
    Ok(().into())
}

#[ewasm_fn]
fn drop_bucket_than_check(
    name: &str,
    remine_buckets: Vec<String>,
) -> anyhow::Result<sewup::primitives::EwasmAny> {
    let mut storage = sewup::kv::Store::load(None)?;
    storage.drop_bucket(name)?;
    storage.commit()?;

    let s = sewup::kv::Store::load(None)?;
    let mut current_buckets = s.buckets();
    current_buckets.sort();
    if current_buckets != remine_buckets {
        return Err(KVError::IncorrectBuckets(current_buckets).into());
    }
    Ok(().into())
}

#[ewasm_fn]
fn new_bucket_with_specific_struct() -> anyhow::Result<sewup::primitives::EwasmAny> {
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
    Ok(().into())
}

#[ewasm_fn]
fn check_objects_in_bucket() -> anyhow::Result<sewup::primitives::EwasmAny> {
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

    Ok(().into())
}

#[ewasm_fn]
fn delete_object_in_bucket() -> anyhow::Result<sewup::primitives::EwasmAny> {
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

    Ok(().into())
}

#[ewasm_fn]
fn non_register_function() -> anyhow::Result<sewup::primitives::EwasmAny> {
    // A function forget to register
    Ok(().into())
}

#[ewasm_main(auto)]
fn main() -> anyhow::Result<sewup::primitives::EwasmAny> {
    use sewup_derive::ewasm_input_from;

    let contract = sewup::primitives::Contract::new()?;

    let output = match contract.get_function_selector()? {
        ewasm_fn_sig!(check_ver_and_feat) => {
            check_ver_and_feat(0, vec![sewup::kv::Feature::Default])?
        }
        ewasm_fn_sig!(check_buckets) => {
            check_buckets(vec!["bucket1".to_string(), "bucket2".to_string()])?
        }
        ewasm_fn_sig!(drop_bucket_than_check) => {
            drop_bucket_than_check("bucket1", vec!["bucket2".to_string()])?
        }
        ewasm_fn_sig!(put_pair_to_bucket1) => ewasm_input_from!(contract move put_pair_to_bucket1)?,
        ewasm_fn_sig!(get_value_to_bucket1) => {
            ewasm_input_from!(contract move get_value_to_bucket1)?
        }

        // Following handler is for other test
        ewasm_fn_sig!(new_bucket_with_specific_struct) => new_bucket_with_specific_struct()?,
        ewasm_fn_sig!(check_objects_in_bucket) => check_objects_in_bucket()?,
        ewasm_fn_sig!(delete_object_in_bucket) => delete_object_in_bucket()?,
        _ => return Err(KVError::UnknownHandle.into()),
    };

    Ok(output)
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_err_output};

    #[ewasm_test]
    fn test_execute_storage_operations() {
        ewasm_assert_eq!(
            non_register_function(),
            ewasm_err_output!(KVError::UnknownHandle)
        );

        ewasm_assert_ok!(check_ver_and_feat());

        ewasm_assert_ok!(check_buckets());

        let input_pair = InputPair(100, vec![1, 2, 3, 4]);
        ewasm_assert_ok!(put_pair_to_bucket1(input_pair));

        ewasm_assert_eq!(
            get_value_to_bucket1(100),
            vec![
                1, 32, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );

        ewasm_assert_ok!(drop_bucket_than_check());
    }

    #[ewasm_test]
    fn test_execute_bucket_operations() {
        ewasm_assert_ok!(new_bucket_with_specific_struct());
        ewasm_assert_ok!(check_objects_in_bucket());
        ewasm_assert_ok!(delete_object_in_bucket());
    }
}
