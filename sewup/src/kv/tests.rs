use std::cell::RefCell;
use std::sync::Arc;

use crate::runtimes::{handler::ContractHandler, test::TestRuntime};

use ethereum_types::Address;
use sewup_derive::fn_sig;
use tempfile::NamedTempFile;

#[test]
fn test_execute_storage_operations() {
    let runtime = Arc::new(RefCell::new(TestRuntime::default()));
    let run_function =
        |fn_name: &str, sig: [u8; 4], input_data: Option<&[u8]>, expect_output: Vec<u8>| {
            let config_file = NamedTempFile::new().unwrap();

            let mut h = ContractHandler {
                sender_address: Address::from_low_u64_be(1),
                call_data: Some(format!(
                    "{}/../resources/test/kv_contract.wasm",
                    env!("CARGO_MANIFEST_DIR")
                )),
                config_file_path: Some(config_file.path().into()),
                ..Default::default()
            };

            h.rt = Some(runtime.clone());

            match h.execute(sig, input_data, 1_000_000) {
                Ok(r) => assert_eq!((fn_name, r.output_data), (fn_name, expect_output)),
                Err(e) => {
                    panic!("vm error: {:?}", e);
                }
            }
        };
    run_function("Commit test", fn_sig!(empty_commit()), None, vec![]);
    run_function(
        "Unknow Handler",
        fn_sig!(unknow_function()),
        None,
        vec![
            117, 110, 107, 110, 111, 119, 32, 104, 97, 110, 100, 108, 101, 114,
        ],
    );
    run_function(
        "Verson check test",
        fn_sig!(check_version_and_features(
            version: u8,
            features: Vec<Feature>
        )),
        None,
        vec![],
    );
    run_function(
        "Check empty storage size",
        fn_sig!(check_empty_storage_size(size: u32)),
        None,
        vec![],
    );
    run_function("Add buckets", fn_sig!(add_buckets()), None, vec![]);
    run_function(
        "Check buckets",
        fn_sig!(check_buckets(buckets: Vec<String>)),
        None,
        vec![],
    );
    run_function(
        "Drop bucket and check",
        fn_sig!(drop_bucket_than_check(
            name: &str,
            remine_buckets: Vec<String>
        )),
        None,
        vec![],
    );
}

#[test]
fn test_execute_bucket_operations() {
    let runtime = Arc::new(RefCell::new(TestRuntime::default()));
    let run_function =
        |fn_name: &str, sig: [u8; 4], input_data: Option<&[u8]>, expect_output: Vec<u8>| {
            let config_file = NamedTempFile::new().unwrap();

            let mut h = ContractHandler {
                sender_address: Address::from_low_u64_be(1),
                call_data: Some(format!(
                    "{}/../resources/test/kv_contract.wasm",
                    env!("CARGO_MANIFEST_DIR")
                )),
                config_file_path: Some(config_file.path().into()),
                ..Default::default()
            };

            h.rt = Some(runtime.clone());

            match h.execute(sig, input_data, 1_000_000) {
                Ok(r) => assert_eq!((fn_name, r.output_data), (fn_name, expect_output)),
                Err(e) => {
                    panic!("vm error: {:?}", e);
                }
            }
        };
    run_function(
        "Init bucket with struct",
        fn_sig!(new_bucket_with_specific_struct()),
        None,
        vec![],
    );
    run_function(
        "Check objects in the bucket",
        fn_sig!(check_objects_in_bucket()),
        None,
        vec![],
    );
    run_function(
        "Check object deletection of bucket",
        fn_sig!(delete_object_in_bucket()),
        None,
        vec![],
    );
}
