//! `kv` feature provides a simple way to key/value store things into evm
//! It aims to be lightweight and with a nice high level interface.
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
mod store;

//TODO: remove this after implement
#[allow(unused_variables)]
#[allow(dead_code)]
mod bucket;
