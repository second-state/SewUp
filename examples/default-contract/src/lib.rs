use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup_derive::{ewasm_fn, ewasm_main, fn_sig, input_from};

mod errors;
use errors::Error;

#[derive(Default, Serialize, Deserialize)]
struct SimpleStruct {
    trust: bool,
    description: String,
}

#[ewasm_fn]
fn check_input_object(s: SimpleStruct) -> Result<()> {
    if !s.trust {
        return Err(Error::NotTrustedInput.into());
    }
    Ok(())
}

#[ewasm_main]
fn main() -> Result<()> {
    let contract = Contract::new()?;
    match contract.get_function_selector()? {
        fn_sig!(check_input_object) => input_from!(contract, check_input_object)?,
        _ => return Err(Error::UnknownHandle.into()),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sewup::bincode;
    use sewup::runtimes::{handler::ContractHandler, test::TestRuntime};
    use sewup_derive::*;
    use std::cell::RefCell;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::Arc;

    fn build_wasm() -> String {
        let output = Command::new("sh")
            .arg("-c")
            .arg("cargo build --release --target=wasm32-unknown-unknown")
            .output()
            .expect("failed to build wasm binary");
        if !output.status.success() {
            panic!("failt to build wasm binary")
        }
        let pkg_name = env!("CARGO_PKG_NAME");
        let base_dir = env!("CARGO_MANIFEST_DIR");
        let wasm_binary = format!(
            "{}/target/wasm32-unknown-unknown/release/{}.wasm",
            base_dir,
            pkg_name.replace("-", "_")
        );

        if !Path::new(&wasm_binary).exists() {
            panic!("wasm binary missing")
        }
        wasm_binary
    }

    #[test]
    fn compile_test() {
        build_wasm();
    }
    #[test]
    fn test_execute_basic_operations() {
        let runtime = Arc::new(RefCell::new(TestRuntime::default()));
        let run_function =
            |fn_name: &str, sig: [u8; 4], input_data: Option<&[u8]>, expect_output: Vec<u8>| {
                // let config_file = NamedTempFile::new().unwrap();

                let mut h = ContractHandler {
                    // sender_address: Address::from_low_u64_be(1),
                    call_data: Some(build_wasm()),
                    // config_file_path: Some(config_file.path().into()),
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
}
