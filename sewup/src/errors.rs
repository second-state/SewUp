use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("calldata is absent")]
    CalldataAbsent,
    #[error("the format of calldata is hexaliteral")]
    CalldataMalformat,
    #[error("the size of contract `{0}` is not correct")]
    ContractSizeError(usize),
    #[error("contract address and call data are both absent")]
    InsufficientContractInfoError,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Resource not found")]
    NotFound,
    #[error("Authorization has been refused for current caller's credential")]
    Unauthorized,
}
