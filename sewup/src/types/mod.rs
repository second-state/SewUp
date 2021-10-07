//! Sewup using Raw and Row as basic unit to storage the data
//! - `Raw` is the storage unit in the contract, which contains 32 bytes.
//! - `Row` is the list structure of `Raw`
//! - `SizedString` is a structure to storage String with fixed number of Row
//!
//! It is easy to convert following types into `Raw` or `Row`:
//! `str`, `&str`, `String`, `&String`, `Vec<u8>`, `[u8]`, `Address`, unsigned integer types

#[cfg(test)]
mod tests;

mod raw;
pub use raw::*;

mod row;
pub use row::*;

pub mod errors;

mod sized_str;
pub use sized_str::*;

mod address;
pub use address::*;
