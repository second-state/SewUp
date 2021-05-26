//! RT Trait
//! This trait help you abstract any VM into application runtime layer,
//! such that you can easily use your VM with SewUp libs.
//!
//! Besides, the VMErrors, VMmessage VMresult are rust style wrarp for
//! evm_error, evm_message and evm_result

use anyhow::Result;
use contract_address::ContractAddress;
use ethereum_types::{Address, H256, U256};
use evmc_sys::evmc_call_kind;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VmError {
    #[error("failure")]
    Failure,
    #[error("revert")]
    Revert,
    #[error("out fo gas")]
    OutOfGas,
    #[error("invalid instruction")]
    InvalidInstruction,
    #[error("undefined instruction")]
    UndefinedInstruction,
    #[error("stack overflow")]
    StackOverflow,
    #[error("stack underflow")]
    StackUnderflow,
    #[error("bad jump destination")]
    BadJumpDestination,
    #[error("invalid memory access")]
    InvalidMemoryAccess,
    #[error("call depth exceeded")]
    CallDepthExceeded,
    #[error("static mode violation")]
    StaticModeViolation,
    #[error("precompile failure")]
    PrecompileFailure,
    #[error("contract validation failure")]
    ContractValidationFailure,
    #[error("argument out of range")]
    ArgumentOutOfRange,
    #[error("wasm unreachable instruction")]
    WasmUnreachableInstruction,
    #[error("wasm trap")]
    WasmTrap,
    #[error("internal error")]
    InternalError,
    #[error("rejected")]
    Rejected,
    #[error("out of memory")]
    OutOfMemory,

    #[error("there shoulbe be a caller(sender) for the message")]
    UnkownCaller,

    /// If the EWASM erorrs are not enough for your VM,
    /// you can use this error type to help you customized you error message
    #[error("`{0}`")]
    CustomizedError(String),
}

// TODO: abstract this, such that this can suitable for other chain than ETH
#[derive(Debug, Default)]
pub struct VMResult {
    pub(crate) gas_left: i64,
    pub(crate) output_data: Vec<u8>,
    pub(crate) create_address: Option<Address>,
}

#[derive(Debug, PartialEq)]
pub enum Flags {
    DEFAULT = 0,
    STATIC = 1,
}

// TODO: abstract this, such that this can suitable for other chain than ETH
#[derive(Debug)]
pub struct VMMessage<'a> {
    pub kind: evmc_call_kind,
    pub flags: Flags,
    pub depth: i32,
    pub gas: i64,
    pub destination: Address,
    pub sender: &'a Address,
    pub input_data: Option<&'a Vec<u8>>,
    pub value: U256,
    pub code: Option<&'a Vec<u8>>,
    pub create2_salt: Option<H256>,
}

#[derive(Debug)]
pub struct VMMessageBuilder<'a> {
    pub kind: evmc_call_kind,
    pub flags: Flags,
    pub depth: i32,
    pub gas: i64,
    pub destination: Option<&'a Address>,
    pub sender: Option<&'a Address>,
    pub input_data: Option<&'a Vec<u8>>,
    pub value: U256,
    pub code: Option<&'a Vec<u8>>,
    pub create2_salt: Option<H256>,
}

impl<'a> VMMessageBuilder<'a> {
    /// The Message will be restricted and do not modify the storage
    #[inline]
    pub fn read_only(mut self) -> Self {
        self.flags = Flags::STATIC;
        self
    }

    #[inline]
    pub fn destination(mut self, addr: &'a Address) -> Self {
        self.destination = Some(addr);
        self
    }

    #[inline]
    pub fn sender(mut self, addr: &'a Address) -> Self {
        self.sender = Some(addr);
        self
    }

    #[inline]
    pub fn build(self) -> Result<VMMessage<'a>> {
        let VMMessageBuilder {
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
        } = self;

        if let Some(sender) = sender {
            let destination = if let Some(destination) = destination {
                destination.clone()
            } else {
                Address::from_low_u64_be(0)
            };
            return Ok(VMMessage {
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
            });
        }
        Err(VmError::UnkownCaller.into())
    }

    /// Use Create2 EVM call with predefined salt
    /// The call help you generate the contract address.
    pub fn create2(mut self, salt: H256) -> Self {
        self.create2_salt = Some(salt);
        self
    }
}

impl Default for VMMessageBuilder<'_> {
    fn default() -> Self {
        Self {
            kind: evmc_call_kind::EVMC_CALL,
            flags: Flags::DEFAULT,
            depth: i32::MAX,
            value: U256::from(0u64),
            gas: 0,
            destination: None,
            sender: None,
            input_data: None,
            code: None,
            create2_salt: None,
        }
    }
}

pub trait RT {
    /// let VM execute the message
    fn execute(&mut self, msg: VMMessage) -> Result<VMResult>;

    /// Deploy contract and return the contract address
    fn deploy(&mut self, msg: VMMessage) -> Result<ContractAddress>;
}
