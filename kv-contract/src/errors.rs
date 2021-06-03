use sewup::kv::Feature;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum KVError {
    #[error("the db version `{0}` is unexpect")]
    UnexpectVersion(u8),
    #[error("features are compatible, current features are: `{0:?}`")]
    IncompatibleFeatures(Vec<Feature>),
    #[error("unknow handle")]
    UnknownHandle,
}
