use std::cell::RefCell;
use std::sync::Arc;

use crate::runtimes::{handler::ContractHandler, test::TestRuntime};
use crate::utils::get_function_signature;

use ethereum_types::Address;
use tempfile::NamedTempFile;

#[test]
fn test_execute_wasm_functions() {
    let runtime = Arc::new(RefCell::new(TestRuntime::default()));
    let _run_function =
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

            let r = h.execute(fn_sig, input_data, 1_000_000).unwrap();

            assert_eq!((fn_name, r.output_data), (fn_name, expect_output));
        };
}
