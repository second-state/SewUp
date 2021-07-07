use std::collections::hash_map::HashMap;
use std::convert::TryInto;
use std::ops::Range;

use crate::rdb::{errors::Error, Deserialize, Feature, Serialize, SerializeTrait};
use crate::utils::storage_index_to_addr;

use anyhow::Result;
use ewasm_api::{storage_load, storage_store};
use tiny_keccak::{Hasher, Keccak};

const RDB_FEATURE: u8 = 1;
const VERSION: u8 = 0;
const CONFIG_ADDR: [u8; 32] = [0; 32];

// One table info is half Raw
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TableInfo {
    sig: [u8; 4],
    range: Range<u32>,
    pub record_size: u32,
}

/// DB is a storage space for an account in a specific block.
/// We can import the storage from a past block, and we only commit the storage
/// into the latest block.
///
/// ### DB Header
/// The fist 32 bytes are reserved as header of the store,
///
/// | 0th            | 1st          | 2nd ~ 3rd         | ... | 28th ~ 31st            |
/// |----------------|--------------|-------------------|-----|------------------------|
/// | Sewup Features | version (BE) | RDB Features (LE) | -   | Size of TableInfo (BE) |
///
/// Base on the features, the storage may have different encoding in to binary
#[derive(Serialize)]
pub struct Db {
    _sewup_feature: u8,
    version: u8,
    _features: u16,
    table_info: Vec<TableInfo>,
}

impl Default for Db {
    fn default() -> Self {
        let mut _features = 0;
        _features |= Feature::Default as u16;

        Self {
            _sewup_feature: 0,
            version: VERSION,
            _features,
            table_info: Vec::new(),
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

    pub fn create_table<T: SerializeTrait + Default + Sized>(&mut self, name: &str) -> Result<()> {
        let default_instance = T::default();
        let info = TableInfo {
            sig: get_table_signature(name),
            record_size: bincode::serialized_size(&default_instance)? as u32,
            ..Default::default()
        };
        self.table_info.push(info);

        Ok(())
    }

    // TODO: Implement like this later
    // pub fn table<T>()
    pub fn table<S: AsRef<str>>(name: S) -> Result<u32> {
        Ok(0)
    }

    pub fn drop_table<S: AsRef<str>>(&mut self, name: S) {
        let sig = get_table_signature(name.as_ref());
        let mut new_table_info = Vec::with_capacity(self.table_info.len() - 1);
        for info in self.table_info.iter() {
            if info.sig != sig {
                new_table_info.push(info.clone());
            }
        }
        self.table_info = new_table_info;
    }

    pub fn table_length(&self) -> usize {
        self.table_info.len()
    }

    pub fn table_info<S: AsRef<str>>(&self, name: S) -> Option<TableInfo> {
        let sig = get_table_signature(name.as_ref());
        for info in self.table_info.iter() {
            if info.sig == sig {
                return Some(info.clone());
            }
        }
        None
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
                u16::from_le_bytes(config[2..4].try_into().expect("load rdb feature fail"));

            let mut table_info_size = u32::from_be_bytes(
                config[28..32]
                    .try_into()
                    .expect("load table info length fail"),
            ) as isize;

            let mut addr: [u8; 32] = [0; 32];
            let mut storage_index = 0;

            while table_info_size > 0 {
                storage_index += 1;
                storage_index_to_addr(storage_index, &mut addr);

                let buffer: [u8; 32] = storage_load(&addr.into()).bytes;
                let info1: TableInfo =
                    bincode::deserialize(&buffer[0..16]).expect("load 1st info from chunk fail");
                db.table_info.push(info1);
                if table_info_size > 1 {
                    let info2: TableInfo = bincode::deserialize(&buffer[16..32])
                        .expect("load 2nd info from chunk fail");
                    db.table_info.push(info2);
                }
                table_info_size = table_info_size - 2;
            }

            Ok(db)
        }
    }

    /// Save to db
    pub fn commit(&self) -> Result<u32> {
        let mut buffer = [0u8; 32];
        RDB_FEATURE.to_be_bytes().swap_with_slice(&mut buffer[0..1]);
        VERSION.to_be_bytes().swap_with_slice(&mut buffer[1..2]);
        self._features
            .to_le_bytes()
            .swap_with_slice(&mut buffer[2..4]);

        let mut len_buffer = self.table_info.len().to_be_bytes();
        len_buffer.swap_with_slice(&mut buffer[28..32]);

        storage_store(&CONFIG_ADDR.into(), &buffer.into());

        let mut addr: [u8; 32] = [0; 32];
        let mut storage_index = 0;

        let mut iter = self.table_info.chunks_exact(2);
        while storage_index * 32 < self.table_info.len() * 16 {
            storage_index += 1;
            storage_index_to_addr(storage_index, &mut addr);

            if let Some(chunk) = iter.next() {
                let mut tables =
                    bincode::serialize(&chunk[0]).expect("serialize 1st info of chunk fail");
                tables.append(
                    &mut bincode::serialize(&chunk[1]).expect("serialize 1st info of chunk fail"),
                );
                let part: [u8; 32] = tables.try_into().unwrap();
                storage_store(&addr.into(), &part.into());
            } else {
                let remainder = iter.remainder();
                let mut tables =
                    bincode::serialize(&remainder[0]).expect("serialize 1st info of chunk fail");
                tables.extend_from_slice(&[0u8; 16]);
                let part: [u8; 32] = tables.try_into().unwrap();
                storage_store(&addr.into(), &part.into());
                break;
            }
        }

        // TODO: fix this when implementing table
        Ok(0)
    }
}

fn get_table_signature(table_name: &str) -> [u8; 4] {
    let mut sig = [0; 4];
    let mut hasher = Keccak::v256();
    hasher.update(table_name.as_bytes());
    hasher.finalize(&mut sig);
    sig
}
