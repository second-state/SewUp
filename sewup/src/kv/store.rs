use std::collections::hash_map::HashMap;
use std::convert::TryInto;

use crate::kv::{
    errors::Error,
    traits::{Key, Value},
};
use crate::utils::storage_index_to_addr;

use super::bucket::{Bucket, RawBucket};
use anyhow::Result;
use ewasm_api::{storage_load, storage_store};

const VERSION: u8 = 1;
const CONFIG_ADDR: [u8; 32] = [0; 32];

#[derive(Debug, PartialEq)]
pub enum Feature {
    Default = 1,
}

type Tenants = HashMap<String, Option<RawBucket>>;

/// Store is a storage space for an account in a specific block.
/// We can import the storage from a past block, and we only commit the storage
/// into the latest block.
///
/// Besides, there may be more than one bucket in store, such that you can
/// easily save different kind of key/value pair in the chain.
///
/// ### Store Header
/// The fist 32 bytes are reserved as header of the store,
///
/// | 0th          | 1st ~ 2nd     | ... | 28th ~ 31st |
/// |--------------|---------------|-----|-------------|
/// | version (BE) | Features (LE) | -   | size (BE)   |
///
/// Base on the features, the storage may have different encoding in to binary
pub struct Store {
    version: u8,
    _features: u16,
    _size: u32,
    tenants: Tenants,
}

impl Default for Store {
    fn default() -> Self {
        let mut _features = 0;
        _features |= Feature::Default as u16;

        Self {
            version: VERSION,
            _features,
            _size: 0,
            tenants: Tenants::default(),
        }
    }
}

impl Store {
    pub fn new() -> Result<Self> {
        Ok(Store::default())
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

    /// Get a list of bucket names
    pub fn buckets(&self) -> Vec<String> {
        self.tenants.keys().map(|k| k.to_string()).collect()
    }

    pub fn bucket<'a, K: Key, V: Default + Clone + Value>(
        &mut self,
        name: &str,
    ) -> Result<Bucket<K, V>> {
        let raw_bucket = if self.tenants.contains_key(name) {
            if let Some(bucket) = self.tenants.get_mut(name).unwrap().take() {
                bucket
            } else {
                return Err(Error::BucketAlreadyOpen.into());
            }
        } else {
            self.tenants.insert(name.into(), None);
            (Vec::new(), Vec::new())
        };
        Ok(Bucket::new(name.into(), raw_bucket))
    }

    pub fn drop_bucket<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        let name = name.as_ref().to_string();
        self.tenants.remove(&name);
        Ok(())
    }

    /// Returns the size on load
    pub fn load_size(&self) -> u32 {
        self._size
    }

    /// Returns the size in bytes
    pub fn size(&self) -> Result<u32> {
        let len =
            bincode::serialized_size(&self.tenants).expect("estimate serialized db size fail");
        Ok(len as u32)
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

            store._features =
                u16::from_le_bytes(config[1..3].try_into().expect("load storage feature fail"));

            let mut bin: Vec<u8> = Vec::new();
            let mut addr: [u8; 32] = [0; 32];
            let mut storage_index = 0;
            store._size =
                u32::from_be_bytes(config[28..32].try_into().expect("load db length fail"));

            for i in 0..(store._size / 32) {
                storage_index += 1;
                storage_index_to_addr(storage_index, &mut addr);
                let buffer: [u8; 32] = storage_load(&addr.into()).bytes;
                bin.extend_from_slice(&buffer);
            }
            storage_index += 1;
            storage_index_to_addr(storage_index, &mut addr);
            let buffer: [u8; 32] = storage_load(&addr.into()).bytes;
            bin.extend_from_slice(&buffer);
            store.tenants = bincode::deserialize(&bin).expect("load db binary fail");

            Ok(store)
        }
    }

    /// Save bucket data back to store
    pub fn save<'a, K: Key, V: Value>(&mut self, bucket: Bucket<K, V>) {
        let Bucket {
            name, raw_bucket, ..
        } = bucket;
        self.tenants.insert(name, Some(raw_bucket));
    }

    /// Save to storage
    pub fn commit(&self) -> Result<u32> {
        for (k, v) in self.tenants.iter() {
            if v.is_none() {
                return Err(Error::BucketNotSync(k.to_string()).into());
            }
        }
        let mut buffer = [0u8; 32];
        VERSION.to_be_bytes().swap_with_slice(&mut buffer[0..1]);
        self._features
            .to_le_bytes()
            .swap_with_slice(&mut buffer[1..3]);

        // TODO: store as really need
        let bin = bincode::serialize(&self.tenants).expect("serialize db binary fail");
        let length = bin.len();

        let mut len_buffer = bin.len().to_be_bytes();
        len_buffer.swap_with_slice(&mut buffer[28..32]);

        storage_store(&CONFIG_ADDR.into(), &buffer.into());

        let mut addr: [u8; 32] = [0; 32];
        let mut storage_index = 0;
        let mut iter = bin.chunks_exact(32);
        while storage_index * 32 < length as usize {
            storage_index += 1;
            storage_index_to_addr(storage_index, &mut addr);

            if let Some(chunk) = iter.next() {
                let part: [u8; 32] = chunk.try_into().unwrap();
                storage_store(&addr.into(), &part.into());
            } else {
                let remainder = iter.remainder();
                storage_index_to_addr(storage_index, &mut addr);
                let mut part = [0u8; 32];
                for i in 0..length & 31 {
                    part[i] = remainder[i];
                }
                storage_store(&addr.into(), &part.into());
                break;
            }
        }
        Ok(length as u32)
    }
}
