//! Sewup - Second state eWasm Utility Program
//! A library to help you sew up your Ethereum project with Rust and just like develop in a common backend.
//!
//! Use the crate with different feature to use the high level api just enable the features you
//! want to use.

#[cfg(feature = "token")]
pub mod token;
#[cfg(feature = "token")]
pub use token::*;

#[cfg(feature = "kv")]
pub mod kv;
#[cfg(feature = "kv")]
pub use kv::*;

pub mod errors;

#[allow(dead_code)]
#[cfg(not(test))]
pub mod primitives;

#[allow(dead_code)]
pub mod utils;

/// SewUp help you build up ewasm
/// The runtime module is used for tesing
#[cfg(test)]
pub mod runtimes;

#[allow(unused_variables)]
pub mod types;
