use std::collections::{hash_map::HashMap, BTreeMap};
use std::convert::TryInto;

use super::bucket::Bucket;
use anyhow::Result;
use ewasm_api::{storage_load, storage_store};

const VERSION: u8 = 1;
const CONFIG_ADDR: [u8; 32] = [0; 32];

#[derive(Debug, PartialEq)]
pub enum Feature {
    Default = 1,
}

type Tenants = HashMap<[u8; 4], BTreeMap<Vec<u8>, Vec<u8>>>;

/// Store is a storage space for an account in a specific block.
/// We can import the storage from a past block, and we only commit the storage
/// into the latest block.
///
/// Besides, there may be more than one bucket in store, such that you can
/// easily save different kind of key/value pair in the chain.
///
/// ### Store Header
/// The fist 32 bytes are reserved as header of the store,
/// | 0th          | 1st ~ 2nd     | ... | 31st |
/// |--------------|---------------|-----|------|
/// | version (BE) | Features (LE) | -   | -    |
/// Base on the features, the storage may have different encoding in to binary
pub struct Store {
    pub version: u8,
    _features: u16,
    tenants: Tenants,
}

impl Default for Store {
    fn default() -> Self {
        let mut _features = 0;
        _features |= Feature::Default as u16;

        Self {
            version: VERSION,
            _features,
            tenants: Tenants::default(),
        }
    }
}

impl Store {
    pub fn new() -> Result<Self> {
        Ok(Store::default())
    }

    pub fn features(&self) -> Vec<Feature> {
        let mut output = Vec::<Feature>::new();
        if self._features & Feature::Default as u16 > 0 {
            output.push(Feature::Default)
        }
        output
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
    /// If not the will import db from the latest block
    pub fn load(block_height: Option<i64>) -> Result<Self> {
        if let Some(_block_height) = block_height {
            unimplemented!();
        } else {
            let mut store = Self::new()?;

            let config: [u8; 32] = storage_load(&CONFIG_ADDR.into()).bytes;
            if VERSION != config[0] {
                // TODO
                panic!("migration not implement")
            }

            store._features = u16::from_le_bytes(config[1..3].try_into().unwrap());

            Ok(store)
        }
    }

    /// Save to storage
    pub fn commit(&self) -> Result<()> {
        let mut config = [0u8; 32];
        VERSION.to_be_bytes().swap_with_slice(&mut config[0..1]);
        self._features
            .to_le_bytes()
            .swap_with_slice(&mut config[1..3]);
        storage_store(&CONFIG_ADDR.into(), &config.into());
        Ok(())
    }
}