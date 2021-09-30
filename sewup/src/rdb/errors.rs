use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Record deleted")]
    RecordDeleted,
    #[error("Record Id not correct, it starts from 1 not zero")]
    RecordIdCorrect,
    #[error("Record is not fixed, and overflowed")]
    RecordNotSized,
    #[error("No record in table")]
    TableIsEmpty,
    #[error("`{0}` did not exist")]
    TableNotExist(String),
}
