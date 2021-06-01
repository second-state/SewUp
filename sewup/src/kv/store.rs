use super::bucket::Bucket;
use anyhow::Result;
use std::collections::{hash_map::HashMap, BTreeMap};

/// Store is a storage space for an account in a specific block.
/// We can import the storage from a past block, and we only commit the storage
/// into the latest block.
///
/// Besides, there may be more than one bucket in store, such that you can
/// easily save different kind of key/value pair in the chain.
#[derive(Default)]
pub struct Store {
    tenants: HashMap<[u8; 4], BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl Store {
    pub fn new() -> Result<Self> {
        Ok(Store::default())
    }

    /// Get a list of bucket names
    pub fn buckets(&self) -> Vec<String> {
        Vec::new()
    }

    pub fn bucket(&self, name: &str) -> Result<Bucket> {
        unimplemented!();
    }

    pub fn drop_bucket<S: AsRef<str>>(&self, name: S) -> Result<()> {
        Ok(())
    }

    /// Returns the size in bytes
    pub fn size(&self) -> Result<u64> {
        Ok(0)
    }

    /// Import the database from the specific block height
    pub fn import(&self, block_height: i64) -> Result<()> {
        Ok(())
    }

    /// Save to storage
    pub fn commit(&self) -> Result<()> {
        Ok(())
    }
}
