//! `rdb` feature provides a simple way to store things with relations into ethereum runtime.

/// DB feature flag to enable the different feature for db
#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(PartialEq)]
pub enum Feature {
    Default = 1,
}

mod db;
pub use db::*;

mod table;
pub use table::*;

pub mod traits;

pub mod errors;
