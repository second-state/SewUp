use sewup::rdb::Feature;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug, PartialEq)]
pub enum RDBError {
    #[error("the DB version `{0}` is unexpected.")]
    UnexpectedVersion(u8),
    #[error("features are compatible, current features are: `{0:?}`.")]
    IncompatibleFeatures(Vec<Feature>),
    #[error("`{0}`")]
    SimpleError(String),
    #[error("unknown handler")]
    UnknownHandle,
}
