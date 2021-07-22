#![feature(allocator_api)]
//! Sewup - Second state eWasm Utility Program
//! A library to help you sew up your Ethereum project with Rust and just like develop in a common backend.
//!
//! Use the crate with different feature to use the high level api just enable the features you
//! want to use.
//! - KV feature helps you develop contract as key value database
//! ```toml
//! sewup = { version = "*", features = ['kv'] }
//! sewup-derive = { version = "*", features = ['kv']  }
//! ```
//! - RDB feature helps you develop contract as rdb database
//! ```toml
//! sewup = { version = "*", features = ['rdb'] }
//! sewup-derive = { version = "*", features = ['rdb']  }
//! ```

/// help you build up you contract to handle tokens (experimental)
#[cfg(feature = "token")]
pub mod token;
#[cfg(feature = "token")]
pub use token::*;

/// help you storage thing as key value object pair
#[cfg(feature = "kv")]
pub mod kv;
#[cfg(feature = "kv")]
pub use kv::*;

/// help you storage thing as records in tables
#[cfg(feature = "rdb")]
pub mod rdb;
#[cfg(feature = "rdb")]
pub use rdb::*;

pub mod errors;

/// The primitvie used in contract
#[cfg(not(test))]
pub mod primitives;

#[allow(dead_code)]
pub mod utils;

/// The run time helps user to setup the contract testing environment
#[cfg(not(target_arch = "wasm32"))]
pub mod runtimes;

/// The basic types for storage in low level, and also easiler to used for bytes and string.
pub mod types;

/// Re-export the ewasm_api
/// these api is low level apis, it is better to keep in a library not in the contract file
#[cfg(target_arch = "wasm32")]
pub use ewasm_api;

pub use anyhow::Result;
pub use bincode;
pub use serde::de::DeserializeOwned;
pub use serde::{Deserialize as DeserializeTrait, Serialize as SerializeTrait};
pub use serde_derive::{Deserialize, Serialize};
pub use serde_value::{to_value, Value};
