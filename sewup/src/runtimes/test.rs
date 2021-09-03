//! A runtime for testing

use crate::runtimes::traits::{Flags, VMMessage, VMResult, VmError, RT};

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;

use anyhow::Result;
use evmc_sys::{evmc_call_kind, evmc_revision, evmc_status_code, evmc_storage_status};
use hex::encode;
use rust_ssvm::{create as create_vm, host::HostContext, EvmcVm};

pub struct TestRuntime {
    pub host: TestHost,
    vm: EvmcVm,
}

impl Default for TestRuntime {
    fn default() -> Self {
        Self {
            host: TestHost::default(),
            vm: create_vm(),
        }
    }
}

impl TestRuntime {
    pub fn set_log_file(self, log_file: String) -> Self {
        Self {
            host: self.host.set_log_file(log_file),
            vm: self.vm,
        }
    }
    pub fn set_host(self, mut host: TestHost) -> Self {
        host.log_file = self.host.log_file;
        Self { host, vm: self.vm }
    }
}

impl RT for TestRuntime {
    fn execute(&mut self, msg: VMMessage) -> Result<VMResult> {
        let VMMessage {
            kind,
            flags,
            depth,
            gas,
            destination,
            sender,
            input_data,
            value,
            code,
            create2_salt,
        } = msg;

        let null_input_data = Vec::<u8>::new();
        let mut v = [0u8; 32];
        value.to_big_endian(&mut v);

        let (output_data, gas_left, status_code) = self.vm.execute(
            &mut self.host,
            evmc_revision::EVMC_FRONTIER,
            kind,
            flags == Flags::Static,
            depth,
            gas,
            &destination.0,
            &sender.0,
            input_data.unwrap_or(&null_input_data),
            &v,
            code.unwrap_or_else(|| input_data.unwrap_or(&null_input_data)),
            &create2_salt.map_or_else(|| [0; 32], |h| h.0),
        );
        match status_code {
            evmc_status_code::EVMC_SUCCESS => Ok(VMResult {
                output_data: output_data.into(),
                gas_left,
                ..Default::default()
            }),
            evmc_status_code::EVMC_FAILURE => Err(VmError::Failure.into()),
            evmc_status_code::EVMC_REVERT => Err(VmError::Revert.into()),
            evmc_status_code::EVMC_OUT_OF_GAS => Err(VmError::OutOfGas.into()),
            evmc_status_code::EVMC_INVALID_INSTRUCTION => Err(VmError::InvalidInstruction.into()),
            evmc_status_code::EVMC_UNDEFINED_INSTRUCTION => {
                Err(VmError::UndefinedInstruction.into())
            }
            evmc_status_code::EVMC_STACK_OVERFLOW => Err(VmError::StackOverflow.into()),
            evmc_status_code::EVMC_STACK_UNDERFLOW => Err(VmError::StackUnderflow.into()),
            evmc_status_code::EVMC_BAD_JUMP_DESTINATION => Err(VmError::BadJumpDestination.into()),
            evmc_status_code::EVMC_INVALID_MEMORY_ACCESS => {
                Err(VmError::InvalidMemoryAccess.into())
            }
            evmc_status_code::EVMC_CALL_DEPTH_EXCEEDED => Err(VmError::CallDepthExceeded.into()),
            evmc_status_code::EVMC_STATIC_MODE_VIOLATION => {
                Err(VmError::StaticModeViolation.into())
            }
            evmc_status_code::EVMC_PRECOMPILE_FAILURE => Err(VmError::PrecompileFailure.into()),
            evmc_status_code::EVMC_CONTRACT_VALIDATION_FAILURE => {
                Err(VmError::ContractValidationFailure.into())
            }
            evmc_status_code::EVMC_ARGUMENT_OUT_OF_RANGE => Err(VmError::ArgumentOutOfRange.into()),
            evmc_status_code::EVMC_WASM_UNREACHABLE_INSTRUCTION => {
                Err(VmError::WasmUnreachableInstruction.into())
            }
            evmc_status_code::EVMC_WASM_TRAP => Err(VmError::WasmTrap.into()),
            evmc_status_code::EVMC_INTERNAL_ERROR => Err(VmError::InternalError.into()),
            evmc_status_code::EVMC_REJECTED => Err(VmError::Rejected.into()),
            evmc_status_code::EVMC_OUT_OF_MEMORY => Err(VmError::OutOfMemory.into()),
        }
    }

    fn deploy(&mut self, msg: VMMessage) -> Result<()> {
        let sender = *msg.sender;
        self.execute(msg)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct TestHost {
    store: HashMap<[u8; 20], HashMap<[u8; 32], [u8; 32]>>,
    balance: HashMap<[u8; 20], [u8; 32]>,
    log_file: Option<String>,
}

impl TestHost {
    pub(crate) fn set_log_file(self, file_name: String) -> Self {
        fs::write(&file_name, "").expect("written log fail");
        Self {
            store: self.store,
            balance: self.balance,
            log_file: Some(file_name),
        }
    }
}

/// Impl methods that developer easily modify the state to setup the test runtime as they want
impl TestHost {
    pub fn set_balance_raw(&mut self, addr: &[u8; 20], balance: [u8; 32]) {
        self.balance.insert(*addr, balance);
    }

    pub fn reset_balance(&mut self, addr: &[u8; 20]) {
        self.balance.insert(*addr, Default::default());
    }

    pub fn set_balance(&mut self, addr: &[u8; 20], x: u128) {
        self.balance.insert(
            *addr,
            [
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                ((x >> 120) & 0xff) as u8,
                ((x >> 112) & 0xff) as u8,
                ((x >> 104) & 0xff) as u8,
                (x >> 96) as u8,
                ((x >> 88) & 0xff) as u8,
                ((x >> 80) & 0xff) as u8,
                ((x >> 72) & 0xff) as u8,
                (x >> 64) as u8,
                ((x >> 56) & 0xff) as u8,
                ((x >> 48) & 0xff) as u8,
                ((x >> 40) & 0xff) as u8,
                (x >> 32) as u8,
                ((x >> 24) & 0xff) as u8,
                ((x >> 16) & 0xff) as u8,
                ((x >> 8) & 0xff) as u8,
                (x & 0xff) as u8,
            ],
        );
    }
}

impl HostContext for TestHost {
    fn account_exists(&mut self, addr: &[u8; 20]) -> bool {
        self.balance.contains_key(addr)
    }

    fn get_storage(&mut self, addr: &[u8; 20], key: &[u8; 32]) -> [u8; 32] {
        if !self.store.contains_key(addr) {
            self.store.insert(*addr, Default::default());
        }

        let store = self.store.get_mut(addr).unwrap();

        match store.get(key) {
            Some(v) => *v,
            None => [0; 32],
        }
    }

    fn set_storage(
        &mut self,
        addr: &[u8; 20],
        key: &[u8; 32],
        value: &[u8; 32],
    ) -> evmc_storage_status {
        if !self.store.contains_key(addr) {
            self.store.insert(*addr, Default::default());
        }

        let store = self.store.get_mut(addr).unwrap();
        store.insert(*key, *value);
        evmc_storage_status::EVMC_STORAGE_MODIFIED
    }

    fn get_balance(&mut self, addr: &[u8; 20]) -> [u8; 32] {
        match self.balance.get(addr) {
            Some(v) => *v,
            None => [0; 32],
        }
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

    #[allow(clippy::type_complexity)]
    fn get_tx_context(&mut self) -> ([u8; 32], [u8; 20], [u8; 20], i64, i64, i64, [u8; 32]) {
        ([0; 32], [0; 20], [0; 20], 0, 0, 0, [0; 32])
    }

    fn get_block_hash(&mut self, number: i64) -> [u8; 32] {
        [0; 32]
    }

    fn emit_log(&mut self, addr: &[u8; 20], topics: &Vec<[u8; 32]>, data: &[u8]) {
        let addr_str = encode(addr);
        let topic_str = topics
            .iter()
            .map(|t| {
                let msg = if let Ok(s) = std::str::from_utf8(t) {
                    if s.len() >= 8 {
                        s[0..8].into()
                    } else {
                        s.into()
                    }
                } else {
                    let t_str = encode(t);
                    format!("{}..{}", &t_str[0..4], &t_str[60..64])
                };
                msg
            })
            .collect::<Vec<_>>()
            .join(",");
        let msg = if let Ok(s) = std::str::from_utf8(data) {
            s.into()
        } else {
            format!("{:?}", data)
        };

        eprintln!(
            "{}..{}|{}|{}",
            &addr_str[0..4],
            &addr_str[36..40],
            topic_str,
            msg
        );

        if let Some(log) = self.log_file.take() {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&log)
                .unwrap();

            writeln!(
                file,
                "{}..{}|{}|{}",
                &addr_str[0..4],
                &addr_str[36..40],
                topic_str,
                msg
            )
            .expect("written log fail");
            self.log_file = Some(log);
        }
    }

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
        (vec![0; 32], gas, [0; 20], evmc_status_code::EVMC_SUCCESS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn test_host_log_function() {
        let mut host = TestHost::default().set_log_file("/tmp/host.log".to_string());
        let addr = [255; 20];
        let topics = vec![[255; 32], [65; 32]];
        let data = vec![0u8, 1u8, 2u8, 3u8, 255u8];
        let readable_data = vec![74u8, 79u8, 86u8, 89u8];
        host.emit_log(&addr, &topics, &data);
        host.emit_log(&addr, &topics, &readable_data);
        assert!(fs::metadata("/tmp/host.log").is_ok());
    }
    #[test]
    fn test_adding_balance() {
        let mut host = TestHost::default();
        let addr = [1; 20];
        host.set_balance(&addr, 1);
        assert_eq!(
            host.get_balance(&addr),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1
            ]
        );
        host.set_balance(&addr, 340282366920938463463374607431768211455);
        assert_eq!(
            host.get_balance(&addr),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255
            ]
        );
    }
}
