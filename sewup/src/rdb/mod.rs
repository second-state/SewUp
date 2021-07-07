//! `rdb` feature provides a simple way to store things with relations into ethereum runtime.

#[derive(Debug, PartialEq)]
pub enum Feature {
    Default = 1,
}

mod db;
pub use db::*;

mod errors;

pub use serde::Serialize as SerializeTrait;
pub use serde_derive::{Deserialize, Serialize};
