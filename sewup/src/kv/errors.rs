use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("bucket already open")]
    BucketAlreadyOpen,
    #[error("bucket `{0}` did not sync, use `safe` before commit")]
    BucketNotSync(String),
}
