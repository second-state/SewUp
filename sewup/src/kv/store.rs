use super::bucket::Bucket;
use anyhow::Result;

pub struct Store {}

impl Store {
    pub fn new() -> Result<Self> {
        Ok(Store {})
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
