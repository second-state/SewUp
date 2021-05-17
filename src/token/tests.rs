use crate::erc20::ERC20ContractHandler;
use crate::errors::ContractError as Error;
use crate::traits::{Flags, VMMessage, RT};
use anyhow::Result;
use ethereum_types::Address;
use evmc_sys::{evmc_call_kind, evmc_revision, evmc_status_code, evmc_storage_status};
use rust_ssvm::{create as create_vm, host::HostContext, EvmcVm};
use std::cell::RefCell;
use std::sync::Arc;
use toml;

use tempfile::NamedTempFile;

struct TestRuntime {
    host: TestHost,
    vm: EvmcVm,
}

impl Default for TestRuntime {
    fn default() -> Self {
        Self {
            host: TestHost {},
            vm: create_vm(),
        }
    }
}

impl RT for TestRuntime {
    fn execute(&mut self, msg: VMMessage) -> Result<()> {
        let VMMessage {
            kind,
            flags,
            depth,
            gas,
            destination,
            sender,
            input_data,
            value,
            create2_salt,
        } = msg;

        let null_input_data = Vec::<u8>::new();
        let mut v = [0u8; 32];
        value.to_big_endian(&mut v);

        self.vm.execute(
            &mut self.host,
            evmc_revision::EVMC_FRONTIER,
            kind,
            flags == Flags::STATIC,
            depth,
            gas,
            &destination.0,
            &sender.0,
            input_data.unwrap_or_else(|| &null_input_data),
            &v,
            input_data.unwrap_or_else(|| &null_input_data),
            &create2_salt.map_or_else(|| [0; 32], |h| h.0),
        );
        Ok(())
    }
}

struct TestHost {}
impl HostContext for TestHost {
    fn account_exists(&mut self, addr: &[u8; 20]) -> bool {
        true
    }

    fn get_storage(&mut self, addr: &[u8; 20], key: &[u8; 32]) -> [u8; 32] {
        [0; 32]
    }

    fn set_storage(
        &mut self,
        addr: &[u8; 20],
        key: &[u8; 32],
        value: &[u8; 32],
    ) -> evmc_storage_status {
        evmc_storage_status::EVMC_STORAGE_UNCHANGED
    }

    fn get_balance(&mut self, addr: &[u8; 20]) -> [u8; 32] {
        [0; 32]
    }

    fn get_code_size(&mut self, addr: &[u8; 20]) -> usize {
        0
    }

    fn get_code_hash(&mut self, addr: &[u8; 20]) -> [u8; 32] {
        [0; 32]
    }

    fn copy_code(
        &mut self,
        addr: &[u8; 20],
        offset: &usize,
        buffer_data: &*mut u8,
        buffer_size: &usize,
    ) -> usize {
        0
    }

    fn selfdestruct(&mut self, addr: &[u8; 20], beneficiary: &[u8; 20]) {}

    fn get_tx_context(
        &mut self,
    ) -> (
        [u8; 32],
        [u8; 20],
        [u8; 20],
        i64,
        i64,
        i64,
        [u8; 32],
        [u8; 32],
    ) {
        ([0; 32], [0; 20], [0; 20], 0, 0, 0, [0; 32], [0; 32])
    }

    fn get_block_hash(&mut self, number: i64) -> [u8; 32] {
        [0; 32]
    }

    fn emit_log(&mut self, addr: &[u8; 20], topics: &Vec<[u8; 32]>, data: &[u8]) {}

    fn call(
        &mut self,
        kind: evmc_call_kind,
        destination: &[u8; 20],
        sender: &[u8; 20],
        value: &[u8; 32],
        input: &[u8],
        gas: i64,
        depth: i32,
        is_static: bool,
        salt: &[u8; 32],
    ) -> (Vec<u8>, i64, [u8; 20], evmc_status_code) {
        (Vec::new(), 0, [0; 20], evmc_status_code::EVMC_SUCCESS)
    }
}

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
        "contract_address = \"0x000000000000000000000000000000000000000f\"\nsender_address = \"0x0000000000000000000000000000000000000001\"\n"
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
        sender_address: Address::from_low_u64_be(1),
        contract_address: None,
        call_data: Some("0xabcd".to_string()),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
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
        sender_address: Address::from_low_u64_be(1),
        contract_address: None,
        call_data: Some("0xabcdefeff".to_string()),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
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
        sender_address: Address::from_low_u64_be(1),
        contract_address: None,
        call_data: Some("0xabcdefeffg".to_string()),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
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
        sender_address: Address::from_low_u64_be(1),
        call_data: Some(format!(
            "{}/resources/test/bad.wasm",
            env!("CARGO_MANIFEST_DIR")
        )),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
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
        sender_address: Address::from_low_u64_be(1),
        call_data: Some(format!(
            "{}/resources/test/erc20.wasm",
            env!("CARGO_MANIFEST_DIR")
        )),
        config_file_path: Some(config_file.path().into()),
        ..Default::default()
    };
    let rt = TestRuntime::default();

    c.rt = Some(Arc::new(RefCell::new(rt)));
    let connect_result = c.connect();
    assert!(connect_result.is_ok());
}
