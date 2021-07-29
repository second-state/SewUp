use sewup::kv::Feature;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug, PartialEq)]
pub enum KVError {
    #[error("the DB version `{0}` is unexpected.")]
    UnexpectedVersion(u8),
    #[error("features are compatible, current features are: `{0:?}`.")]
    IncompatibleFeatures(Vec<Feature>),
    #[error("current DB size is `{0}`.")]
    UnexpectedDBSize(u32),
    #[error("current bucket are: `{0:?}.`")]
    IncorrectBuckets(Vec<String>),
    #[error("bucket error: `{0}.`")]
    BucketError(String),
    #[error("current value is `{0}.`")]
    ValueError(String),
    #[error("value not found")]
    ValueNotFound,
    #[error("unknown handler")]
    UnknownHandle,
}
