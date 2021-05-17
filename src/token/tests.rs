use crate::erc20::ERC20ContractHandler;
use crate::errors::ContractError as Error;
use ethereum_types::H160;
use toml;

use tempfile::NamedTempFile;

#[test]
fn test_config_serde() {
    let c1 = ERC20ContractHandler {
        address: Some(H160::from_low_u64_be(15)),
        call_data: Some("0x12345678".into()),
        ..Default::default()
    };
    assert_eq!(
        toml::to_string(&c1).unwrap(),
        "address = \"0x000000000000000000000000000000000000000f\"\n"
    );

    let c2: ERC20ContractHandler =
        toml::from_str("address = \"0x000000000000000000000000000000000000000f\"\n").unwrap();
    assert_eq!(c2.address, c1.address);

    let c3: ERC20ContractHandler = toml::from_str("call_data = \"0x12345678\"\n").unwrap();
    assert_eq!(c3.address, None);
    assert_eq!(c3.call_data.unwrap(), "0x12345678".to_string());
}

#[test]
fn test_handle_error_missing_call_data_and_contract_address() {
    let config_file = NamedTempFile::new().unwrap();
    let mut c = ERC20ContractHandler {
        address: None,
        call_data: None,
        config_file_path: Some(config_file.path().into()),
    };
    let connect_result = c.connect();
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
        address: None,
        call_data: Some("0xabcd".to_string()),
        config_file_path: Some(config_file.path().into()),
    };
    let connect_result = c.connect();
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
        address: None,
        call_data: Some("0xabcdefeff".to_string()),
        config_file_path: Some(config_file.path().into()),
    };
    let connect_result = c.connect();
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
        address: None,
        call_data: Some("0xabcdefeffg".to_string()),
        config_file_path: Some(config_file.path().into()),
    };
    let connect_result = c.connect();
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
        address: None,
        call_data: Some(format!(
            "{}/resources/test/bad.wasm",
            env!("CARGO_MANIFEST_DIR")
        )),
        config_file_path: Some(config_file.path().into()),
    };
    let connect_result = c.connect();
    assert!(connect_result.is_err());
    if let Err(error) = connect_result {
        assert_eq!(
            error.downcast_ref::<Error>().unwrap(),
            &Error::ContractSizeError(0),
        );
    }
}

#[test]
fn test_handle_from_call_data_file() {
    let config_file = NamedTempFile::new().unwrap();

    let mut c = ERC20ContractHandler {
        address: None,
        call_data: Some(format!(
            "{}/resources/test/erc20.wasm",
            env!("CARGO_MANIFEST_DIR")
        )),
        config_file_path: Some(config_file.path().into()),
    };
    let connect_result = c.connect();
    assert!(connect_result.is_ok());
}
