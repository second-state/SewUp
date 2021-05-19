//! A runtime for testing

use crate::traits::{Flags, VMMessage, VMResult, RT};

use anyhow::Result;
use evmc_sys::{evmc_call_kind, evmc_revision, evmc_status_code, evmc_storage_status};
use rust_ssvm::{create as create_vm, host::HostContext, EvmcVm};

pub struct TestRuntime {
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
        Ok(VMResult::default())
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
        evmc_storage_status::EVMC_STORAGE_MODIFIED
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
        (vec![0; 32], gas, [0; 20], evmc_status_code::EVMC_SUCCESS)
    }
}
