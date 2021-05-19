use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("the size of contract `{0}` is not correct")]
    ContractSizeError(usize),
    #[error("contract address and call data are both absent")]
    InsufficientContractInfoError,
    #[error("the format of calldata is hexaliteral")]
    CalldataMalformat,
}
