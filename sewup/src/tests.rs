use std::cell::RefCell;
use std::sync::Arc;

use crate::runtimes::{handler::ContractHandler, test::TestRuntime};

use ethereum_types::Address;
use serde_derive::{Deserialize, Serialize};
use sewup_derive::fn_sig;
use tempfile::NamedTempFile;

#[derive(Default, Serialize, Deserialize)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[test]
fn test_execute_basic_operations() {
    let runtime = Arc::new(RefCell::new(TestRuntime::default()));
    let run_function =
        |fn_name: &str, sig: [u8; 4], input_data: Option<&[u8]>, expect_output: Vec<u8>| {
            let config_file = NamedTempFile::new().unwrap();

            let mut h = ContractHandler {
                sender_address: Address::from_low_u64_be(1),
                call_data: Some(format!(
                    "{}/../resources/test/default_contract.wasm",
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

    let mut simple_struct = SimpleStruct::default();
    let mut bin = bincode::serialize(&simple_struct).unwrap();
    run_function(
        "Check input object",
        fn_sig!(check_input_object(s: SimpleStruct)),
        Some(&bin),
        vec![
            110, 111, 116, 32, 116, 114, 117, 115, 116, 32, 105, 110, 112, 117, 116,
        ],
    );

    simple_struct.trust = true;
    bin = bincode::serialize(&simple_struct).unwrap();
    run_function(
        "Check input object",
        fn_sig!(check_input_object(s: SimpleStruct)),
        Some(&bin),
        vec![],
    );
}
