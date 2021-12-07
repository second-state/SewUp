//! RT Trait
//! This trait help you abstract any VM into application runtime layer,
//! such that you can easily use your VM with SewUp libs.
//!
//! Besides, the VMErrors, VMmessage VMresult are rust style wrarp for
//! evm_error, evm_message and evm_result

use anyhow::Result;
use evmc_sys::evmc_call_kind;
use thiserror::Error;

use crate::types::Raw;

#[remain::sorted]
#[derive(Error, Debug, PartialEq)]
pub enum VmError {
    #[error("argument out of range")]
    ArgumentOutOfRange,
    #[error("bad jump destination")]
    BadJumpDestination,
    #[error("call depth exceeded")]
    CallDepthExceeded,
    #[error("contract validation failure")]
    ContractValidationFailure,
    /// If the EWASM errors are not enough for your VM,
    /// you can use this error type to help you customized you error message
    #[error("`{0}`")]
    CustomizedError(String),
    #[error("failure")]
    Failure,
    #[error("internal error")]
    InternalError,
    #[error("invalid instruction")]
    InvalidInstruction,
    #[error("invalid memory access")]
    InvalidMemoryAccess,
    #[error("out of gas")]
    OutOfGas,
    #[error("out of memory")]
    OutOfMemory,
    #[error("precompile failure")]
    PrecompileFailure,
    #[error("rejected")]
    Rejected,
    #[error("revert")]
    Revert,
    #[error("stack overflow")]
    StackOverflow,
    #[error("stack underflow")]
    StackUnderflow,
    #[error("static mode violation")]
    StaticModeViolation,
    #[error("undefined instruction")]
    UndefinedInstruction,
    #[error("there shoulbe be a caller(sender) for the message")]
    UnknownCaller,
    #[error("wasm trap")]
    WasmTrap,
    #[error("wasm unreachable instruction")]
    WasmUnreachableInstruction,
}

// TODO: abstract this, such that this can suitable for other chain than ETH
#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(Default)]
pub struct VMResult {
    pub(crate) gas_left: i64,
    pub output_data: Vec<u8>,
    pub(crate) create_address: Option<Raw>,
}

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(PartialEq)]
pub enum Flags {
    Default = 0,
    Static = 1,
}

// TODO: abstract this, such that this can suitable for other chain than ETH
#[cfg_attr(any(feature = "debug", test), derive(Debug))]
pub struct VMMessage<'a> {
    pub kind: evmc_call_kind,
    pub flags: Flags,
    pub depth: i32,
    pub gas: i64,
    pub destination: Raw,
    pub sender: &'a Raw,
    pub input_data: Option<&'a Vec<u8>>,
    pub value: Raw,
    pub code: Option<&'a Vec<u8>>,
    pub create2_salt: Option<()>,
}

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
pub struct VMMessageBuilder<'a> {
    pub kind: evmc_call_kind,
    pub flags: Flags,
    pub depth: i32,
    pub gas: i64,
    pub destination: Option<&'a Raw>,
    pub sender: Option<&'a Raw>,
    pub input_data: Option<&'a Vec<u8>>,
    pub value: Raw,
    pub code: Option<&'a Vec<u8>>,
    pub create2_salt: Option<()>,
}

impl<'a> VMMessageBuilder<'a> {
    /// The Message will be restricted and do not modify the storage
    #[inline]
    pub fn read_only(mut self) -> Self {
        self.flags = Flags::Static;
        self
    }

    #[inline]
    pub fn destination(mut self, addr: &'a Raw) -> Self {
        self.destination = Some(addr);
        self
    }

    #[inline]
    pub fn sender(mut self, addr: &'a Raw) -> Self {
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
                *destination
            } else {
                Raw::from(0u32)
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
        Err(VmError::UnknownCaller.into())
    }

    /// Use Create2 EVM call with predefined salt
    /// The call help you generate the contract address.
    pub fn create2(self, _salt: ()) -> Self {
        unimplemented!()
    }
}

impl Default for VMMessageBuilder<'_> {
    fn default() -> Self {
        Self {
            kind: evmc_call_kind::EVMC_CALL,
            flags: Flags::Default,
            depth: i32::MAX,
            value: Raw::from(0u32),
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

    /// Deploy contract
    fn deploy(&mut self, msg: VMMessage) -> Result<()>;
}
