//! A runtime for testing

use crate::runtimes::traits::{Flags, VMMessage, VMResult, VmError, RT};

use contract_address::ContractAddress;
use ethereum_types::U256;

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
            flags == Flags::STATIC,
            depth,
            gas,
            &destination.0,
            &sender.0,
            input_data.unwrap_or_else(|| &null_input_data),
            &v,
            code.unwrap_or(input_data.unwrap_or_else(|| &null_input_data)),
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

    fn deploy(&mut self, msg: VMMessage) -> Result<ContractAddress> {
        let sender = msg.sender.clone();
        self.execute(msg)?;
        Ok(ContractAddress::from_sender_and_nonce(
            &sender,
            &U256::default(),
        ))
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
