use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RDBError {
    #[error("unknown handler")]
    UnknownHandle,
}
