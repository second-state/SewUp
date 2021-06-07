use std::cell::RefCell;
use std::sync::Arc;

use crate::runtimes::{handler::ContractHandler, test::TestRuntime};
use crate::utils::get_function_signature;

use ethereum_types::Address;
use tempfile::NamedTempFile;

#[test]
fn test_execute_wasm_functions() {
    let runtime = Arc::new(RefCell::new(TestRuntime::default()));
    let run_function =
        |fn_name: &str, fn_sig: [u8; 4], input_data: Option<&[u8]>, expect_output: Vec<u8>| {
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

            match h.execute(fn_sig, input_data, 1_000_000) {
                Ok(r) => assert_eq!((fn_name, r.output_data), (fn_name, expect_output)),
                Err(e) => {
                    panic!("vm error: {:?}", e);
                }
            }
        };
    run_function(
        "Commit test",
        get_function_signature("empty_commit()"),
        None,
        vec![],
    );
    run_function(
        "Unknow Handler",
        get_function_signature("unknow_function()"),
        None,
        vec![
            117, 110, 107, 110, 111, 119, 32, 104, 97, 110, 100, 108, 101, 114,
        ],
    );
    run_function(
        "Verson check test",
        get_function_signature("check_version_and_features(u8,Vec<Feature>)"),
        None,
        vec![],
    );
    run_function(
        "Check empty storage size",
        get_function_signature("check_empty_storage_size(u32)"),
        None,
        vec![],
    );
    run_function(
        "Add buckets",
        get_function_signature("add_buckets()"),
        None,
        vec![],
    );
    run_function(
        "Check buckets",
        get_function_signature("check_buckets(Vec<String>)"),
        None,
        vec![],
    );
    run_function(
        "Drop bucket and check",
        get_function_signature("drop_bucket_than_check(&str,Vec<String>)"),
        None,
        vec![],
    );
}
