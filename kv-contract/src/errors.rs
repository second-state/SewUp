use sewup::kv::Feature;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum KVError {
    #[error("the db version `{0}` is unexpect.")]
    UnexpectVersion(u8),
    #[error("features are compatible, current features are: `{0:?}`.")]
    IncompatibleFeatures(Vec<Feature>),
    #[error("current db size is `{0}`.")]
    UnexpectedDBSize(u32),
    #[error("current bucket are: `{0:?}.`")]
    IncorrectBuckets(Vec<String>),
    #[error("bucket error: `{0}.`")]
    BucketError(String),
    #[error("unknow handler")]
    UnknownHandle,
}
