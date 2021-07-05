use std::collections::hash_map::HashMap;
use std::convert::TryInto;

use crate::rdb::{errors::Error, Feature, Serialize};
use crate::utils::storage_index_to_addr;

use anyhow::Result;
use ewasm_api::{storage_load, storage_store};

const RDB_FEATURE: u8 = 1;
const VERSION: u8 = 0;
const CONFIG_ADDR: [u8; 32] = [0; 32];

/// DB is a storage space for an account in a specific block.
/// We can import the storage from a past block, and we only commit the storage
/// into the latest block.
///
/// ### DB Header
/// The fist 32 bytes are reserved as header of the store,
///
/// | 0th            | 1st          | 2nd ~ 3rd         | ... |
/// |----------------|--------------|-------------------|-----|
/// | Sewup Features | version (BE) | RDB Features (LE) | -   |
///
/// Base on the features, the storage may have different encoding in to binary
#[derive(Serialize)]
pub struct Db {
    _sewup_feature: u8,
    version: u8,
    _features: u16,
}

impl Default for Db {
    fn default() -> Self {
        let mut _features = 0;
        _features |= Feature::Default as u16;

        Self {
            _sewup_feature: 0,
            version: VERSION,
            _features,
        }
    }
}

impl Db {
    pub fn new() -> Result<Self> {
        Ok(Db::default())
    }

    /// The version of current storage
    #[inline]
    pub fn version(&self) -> u8 {
        self.version
    }

    /// The feature enabled in current storage
    pub fn features(&self) -> Vec<Feature> {
        let mut output = Vec::<Feature>::new();
        if self._features & Feature::Default as u16 > 0 {
            output.push(Feature::Default)
        }
        output
    }

    pub fn table(&mut self, name: &str) -> Result<()> {
        Ok(())
    }

    pub fn drop_table<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        Ok(())
    }

    /// Import the database from the specific block height
    /// If not the will import db from the latest block
    pub fn load(block_height: Option<i64>) -> Result<Self> {
        if let Some(_block_height) = block_height {
            unimplemented!();
        } else {
            let mut db = Self::new()?;

            let config: [u8; 32] = storage_load(&CONFIG_ADDR.into()).bytes;

            if RDB_FEATURE != config[0] {
                panic!("Sewup feature not correct")
            }

            if VERSION != config[1] {
                // TODO
                panic!("migration not implement")
            }

            db._features =
                u16::from_le_bytes(config[1..3].try_into().expect("load rdb feature fail"));

            let mut bin: Vec<u8> = Vec::new();
            let mut addr: [u8; 32] = [0; 32];

            Ok(db)
        }
    }

    /// Save to storage
    pub fn commit(&self) -> Result<u32> {
        let mut buffer = [0u8; 32];
        RDB_FEATURE.to_be_bytes().swap_with_slice(&mut buffer[0..1]);
        VERSION.to_be_bytes().swap_with_slice(&mut buffer[1..2]);
        self._features
            .to_le_bytes()
            .swap_with_slice(&mut buffer[2..4]);

        // TODO: store as really need
        let bin = bincode::serialize(&self).expect("serialize db binary fail");
        let length = bin.len();

        storage_store(&CONFIG_ADDR.into(), &buffer.into());

        Ok(length as u32)
    }
}
