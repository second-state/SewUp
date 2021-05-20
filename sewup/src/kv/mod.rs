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
//! let mut connect_string = "sewup://sender_address@node_ip:node_port/kv_contract_config_file_path";
//! // or sewup://default_sender_address@node_ip:node_port/kv_contract_address
//!
//! // Connect to evm, open kv store
//! let store = Store::new::<Raw, Raw>(connect_string)?;
//!
//! // Set testing = 123
//! store.set(b"test", b"123")?;
//! assert!(store.get(b"test").unwrap().unwrap() == "123");
//! assert!(store.get(b"not exist").unwrap() == None);
//!
//! // Change user
//! let store2 = store.change_user::<Integer, String>(sernder2_address)?;
//! store2.set(1, "Testing");
//! ```
//!
//! These serialization features will be support
//! 1. msgpack
//! 2. bincode
//! 3. json
