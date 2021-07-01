use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("not trust input")]
    NotTrustedInput,
    #[error("unknown handler")]
    UnknownHandle,
}
