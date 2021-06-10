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
//! use sewup::kv::*;
//!
//! let store = Store::new()?;
//! let bucket = Store.bucket::<Raw, Raw>("default")?;
//!
//! // Set testing = 123
//! bucket.set(b"test", b"123")?;
//! assert!(bucket.get(b"test").unwrap().unwrap() == "123");
//! assert!(bucket.get(b"not exist").unwrap() == None);
//!
//! // Set store with specific types
//! let bucket2 = Store.bucket::<Integer, String>("bucket2")?;
//! bucket2.set(1, "Testing");
//! ```
//!
//! These serialization features will be support
//! 1. msgpack
//! 2. bincode
//! 3. json

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

mod traits;

#[allow(dead_code)]
mod errors;
