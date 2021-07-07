//! `rdb` feature provides a simple way to store things with relations into ethereum runtime.

#[derive(Debug, PartialEq)]
pub enum Feature {
    Default = 1,
}

#[allow(unused_variables)]
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
mod db;
#[cfg(target_arch = "wasm32")]
pub use db::*;
#[cfg(not(target_arch = "wasm32"))]
pub struct Db {}

mod errors;

pub use serde::Serialize as SerializeTrait;
pub use serde_derive::{Deserialize, Serialize};
