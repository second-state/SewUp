use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeployError {
    #[error("[deploy] section is incorrect in Cargo.toml")]
    ConfigIncorrect,
}
