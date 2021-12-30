//! `kv` feature provides a simple way to key/value store things into ethereum runtime.
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
//TODO: run this doc test with target wasm32
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

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(PartialEq)]
pub enum Feature {
    Default = 1,
}

#[allow(unused_variables)]
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
mod store;
#[cfg(target_arch = "wasm32")]
pub use store::*;
#[cfg(not(target_arch = "wasm32"))]
pub struct Store {}

#[allow(unused_variables)]
#[allow(dead_code)]
mod bucket;
pub use bucket::*;

#[cfg(test)]
mod tests;

pub mod traits;

#[allow(dead_code)]
mod errors;
