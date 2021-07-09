use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("`{0}` did not exist")]
    TableNotExist(String),
    #[error("No record in table")]
    TableIsEmpty,
    #[error("Record is not fixed, and overflowed")]
    RecordNotSized,
    #[error("Record Id not correct, it starts from 1 not zero")]
    RecordIdCorrect,
    #[error("Record deleted")]
    RecordDeleted,
}
