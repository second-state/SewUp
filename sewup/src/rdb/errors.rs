use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("`{0}` did not exist")]
    TableNotExist(String),
}
