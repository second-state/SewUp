use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TypeError {
    #[error("data size excess the limitation `{0}`")]
    SizeExcess(usize),
}
