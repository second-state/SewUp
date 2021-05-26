use std::cell::RefCell;
use std::io::Read;
use std::sync::Arc;

mod handler;

use crate::errors::ContractError as Error;
use crate::runtimes::test::TestRuntime;
use crate::token::signature;

use handler::ERC20ContractHandler;

use ethereum_types::Address;
use toml;

use tempfile::NamedTempFile;

#[test]
fn test_config_serde() {
    let c1 = ERC20ContractHandler {
        contract_address: Some(Address::from_low_u64_be(15)),
        sender_address: Address::from_low_u64_be(1),
        call_data: Some("0x12345678".into()),
        ..Default::default()
    };
    assert_eq!(
        toml::to_string(&c1).unwrap(),
        "contract_address = \"0x000000000000000000000000000000000000000f\"\ncall_data = \"0x12345678\"\n"
    );

    let c2: ERC20ContractHandler =
        toml::from_str("contract_address = \"0x000000000000000000000000000000000000000f\"\nsender_address = \"0x0000000000000000000000000000000000000001\"\n").unwrap();
    assert_eq!(c2.contract_address, c1.contract_address);

    let c3: ERC20ContractHandler = toml::from_str("call_data = \"0x12345678\"\nsender_address = \"0x0000000000000000000000000000000000000001\"\n").unwrap();
    assert_eq!(c3.contract_address, None);
    assert_eq!(c3.call_data.unwrap(), "0x12345678".to_string());
}

#[test]
fn test_handle_error_missing_call_data_and_contract_address() {
    let config_file = NamedTempFile::new().unwrap();
    let mut c = ERC20ContractHandler {
        sender_address: Address::from_low_u64_be(1),
        contract_address: None,
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
    };
    let connect_result = c.connect(10000);
    assert!(connect_result.is_err());
    if let Err(error) = connect_result {
        assert_eq!(
            error.downcast_ref::<Error>().unwrap(),
            &Error::InsufficientContractInfoError
        );
    }
}

#[test]
fn test_handle_error_for_small_call_data() {
    let config_file = NamedTempFile::new().unwrap();
    let mut c = ERC20ContractHandler {
        sender_address: Address::from_low_u64_be(1),
        contract_address: None,
        call_data: Some("0xabcd".to_string()),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
    };
    let connect_result = c.connect(10000);
    assert!(connect_result.is_err());
    if let Err(error) = connect_result {
        assert_eq!(
            error.downcast_ref::<Error>().unwrap(),
            &Error::ContractSizeError(2),
        );
    }
}

#[test]
fn test_handle_error_for_odd_size_call_data() {
    let config_file = NamedTempFile::new().unwrap();
    let mut c = ERC20ContractHandler {
        sender_address: Address::from_low_u64_be(1),
        contract_address: None,
        call_data: Some("0xabcdefeff".to_string()),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
    };
    let connect_result = c.connect(10000);
    assert!(connect_result.is_err());
    if let Err(error) = connect_result {
        assert_eq!(
            error.downcast_ref::<Error>().unwrap(),
            &Error::CalldataMalformat,
        );
    }
}

#[test]
fn test_handle_error_for_mal_call_data() {
    let config_file = NamedTempFile::new().unwrap();
    let mut c = ERC20ContractHandler {
        sender_address: Address::from_low_u64_be(1),
        contract_address: None,
        call_data: Some("0xabcdefeffg".to_string()),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
    };
    let connect_result = c.connect(10000);
    assert!(connect_result.is_err());
    if let Err(error) = connect_result {
        assert_eq!(
            error.downcast_ref::<Error>().unwrap(),
            &Error::CalldataMalformat,
        );
    }
}

#[test]
fn test_handle_error_for_mal_call_data_file() {
    let config_file = NamedTempFile::new().unwrap();

    let mut c = ERC20ContractHandler {
        sender_address: Address::from_low_u64_be(1),
        call_data: Some(format!(
            "{}/../resources/test/bad.wasm",
            env!("CARGO_MANIFEST_DIR")
        )),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
    };
    let connect_result = c.connect(10000);
    assert!(connect_result.is_err());
    if let Err(error) = connect_result {
        assert_eq!(
            error.downcast_ref::<Error>().unwrap(),
            &Error::ContractSizeError(0),
        );
    }
}

#[test]
fn test_deploy_wasm() {
    let mut config_file = NamedTempFile::new().unwrap();

    let mut h = ERC20ContractHandler {
        sender_address: Address::from_low_u64_be(1),
        call_data: Some(format!(
            "{}/../resources/test/erc20_contract.wasm",
            env!("CARGO_MANIFEST_DIR")
        )),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
    };

    h.rt = Some(Arc::new(RefCell::new(TestRuntime::default())));

    let connect_result = h.connect(1_000_000);
    assert!(connect_result.is_ok());

    let mut buf = String::new();
    config_file.read_to_string(&mut buf).unwrap();
    assert_eq!(
        buf,
        "contract_address = \"0x522b3294e6d06aa25ad0f1b8891242e335d3b459\"\n"
    );
}

#[test]
fn test_execute_wasm_functions() {
    fn run_function(fun_sig: [u8; 4], input_data: Option<&[u8]>, expect_ouput: Vec<u8>) {
        let config_file = NamedTempFile::new().unwrap();

        let mut h = ERC20ContractHandler {
            sender_address: Address::from_low_u64_be(1),
            call_data: Some(format!(
                "{}/../resources/test/erc20_contract.wasm",
                env!("CARGO_MANIFEST_DIR")
            )),
            config_file_path: Some(config_file.path().into()),
            ..Default::default()
        };

        h.rt = Some(Arc::new(RefCell::new(TestRuntime::default())));

        let r = h.execute(fun_sig, input_data, 1_000_000).unwrap();

        assert_eq!(r.output_data, expect_ouput);
    }
    run_function(
        signature::NAME_SIGNATURE,
        None,
        vec![
            69, 82, 67, 50, 48, 84, 111, 107, 101, 110, 68, 101, 109, 111,
        ],
    );
    run_function(signature::SYMBOL_SIGNATURE, None, vec![69, 84, 68]);
    run_function(
        signature::DECIMALS_SIGNATURE,
        None,
        vec![0, 0, 0, 0, 0, 0, 0, 0],
    );
    run_function(
        signature::TOTAL_SUPPLY_SIGNATURE,
        None,
        vec![0, 0, 0, 0, 5, 245, 225, 0],
    );
    // TODO: check why this fail
    // run_function(signature::MINT_SIGNATURE, None, vec![]);
}
