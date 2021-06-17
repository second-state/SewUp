//! `kv` feature provides a simple way to key/value store things into evm
//! It aims to be lightweight and with a nice high level interface.
//! Store is an abstract storage instance from one account in one block
//! There can be more than one bucket in a store.
//! Besides, store can improt data from the specific block.
//! Different bucket can defined different kind of key value storage pair.
//!
//! Please check out the structure `kv::Store` and `kv::Bucket` to learn more about this.
//!
//!
//! ## Getting started
//!
//! Add follow sewup with `kv` feature enabled.
//! > sewup = { features = ["kv"] }
//!
//! ```ignore
//! use sewup::kv::Store;
//! use sewup::types::{Raw, Row};
//!
//! let mut store = Store::new().unwrap();
//! let mut bucket = store.bucket::<Raw, Raw>("default").unwrap();
//!
//! // Set testing = 123
//! bucket.set(b"test".into(), b"123".into());
//!
//! // Set store with specific types
//! let mut bucket2 = store.bucket::<Raw, Row>("bucket2").unwrap();
//! bucket2.set(b"long".into(), "Testing".to_string().into());
//! ```

//TODO: remove this after implement
#[allow(unused_variables)]
#[allow(dead_code)]
// This will be test after compiled into wasm
#[cfg(not(test))]
mod store;
#[cfg(not(test))]
pub use store::*;

//TODO: remove this after implement
#[allow(unused_variables)]
#[allow(dead_code)]
// This will be test after compiled into wasm
#[cfg(not(test))]
mod bucket;
#[cfg(not(test))]
pub use bucket::*;

#[cfg(test)]
mod tests;

pub mod traits;

#[allow(dead_code)]
mod errors;
