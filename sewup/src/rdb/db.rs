use std::convert::TryInto;
use std::marker::PhantomData;
use std::ops::Range;

use crate::rdb::table::Table;
use crate::rdb::traits::{Record, HEADER_SIZE};
use crate::rdb::{errors::Error, Deserialize, Feature, Serialize, SerializeTrait};
use crate::utils::storage_index_to_addr;

use anyhow::Result;
#[cfg(target_arch = "wasm32")]
use ewasm_api::{storage_load, storage_store};
use tiny_keccak::{Hasher, Keccak};

#[cfg(target_arch = "wasm32")]
const RDB_FEATURE: u8 = 1;
const VERSION: u8 = 0;
#[cfg(target_arch = "wasm32")]
const CONFIG_ADDR: [u8; 32] = [0; 32];

pub type TableSig = [u8; 4];

// One table info is half Raw
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct TableInfo {
    pub(crate) sig: TableSig,
    pub range: Range<u32>,
    pub record_raw_size: u32,
}

/// DB is a storage space for an account in a specific block.
/// We can import the storage from a past block, and we only commit the storage
/// into the latest block.
///
/// ### DB Header
/// The fist 32 bytes are reserved as header of the store,
///
/// | 0th            | 1st          | 2nd ~ 3rd         | ... | 28th ~ 31st              |
/// |----------------|--------------|-------------------|-----|--------------------------|
/// | Sewup Features | version (BE) | RDB Features (LE) | -   | length of TableInfo (BE) |
///
/// Base on the features, the storage may have different encoding in to binary
#[derive(Serialize)]
pub struct Db {
    _sewup_feature: u8,
    version: u8,
    _features: u16,
    pub(crate) table_info: Vec<TableInfo>,
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

    pub fn create_table<T: SerializeTrait + Default + Sized + Record>(&mut self) -> Result<()> {
        let default_instance = T::default();
        let ser_size = bincode::serialized_size(&default_instance)?;
        let record_raw_size = if ser_size == 0 {
            0u32
        } else {
            (ser_size as u32 + HEADER_SIZE) / 32 + 1
        };
        let info = if self.table_info.is_empty() {
            TableInfo {
                sig: get_table_signature(std::any::type_name::<T>()),
                record_raw_size,
                range: (2..2),
            }
        } else {
            let TableInfo {
                range: last_table_range,
                ..
            } = self.table_info[self.table_info.len() - 1].clone();
            TableInfo {
                sig: get_table_signature(std::any::type_name::<T>()),
                record_raw_size,
                range: (last_table_range.end..last_table_range.end),
            }
        };
        self.table_info.push(info);

        Ok(())
    }

    pub fn table<T: SerializeTrait + Default + Sized + Record>(self) -> Result<Table<T>> {
        let info = self
            .table_info::<T>()
            .ok_or(Error::TableNotExist(std::any::type_name::<T>().into()))?;
        Ok(Table::<T> {
            info,
            data: Vec::new(),
            phantom: PhantomData,
        }
        .load_data()?)
    }

    pub fn drop_table<T>(&mut self) {
        let sig = get_table_signature(std::any::type_name::<T>());
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

    pub fn table_info<T>(&self) -> Option<TableInfo> {
        let sig = get_table_signature(std::any::type_name::<T>());
        for info in self.table_info.iter() {
            if info.sig == sig {
                return Some(info.clone());
            }
        }
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(block_height: Option<i64>) -> Result<Self> {
        unimplemented!()
    }

    /// Import the database from the specific block height
    /// If not the will import db from the latest block
    #[cfg(target_arch = "wasm32")]
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
                let mut info =
                    bincode::deserialize(&buffer[0..16]).expect("load 1st info from chunk fail");
                db.table_info.push(info);
                if table_info_size > 1 {
                    info = bincode::deserialize(&buffer[16..32])
                        .expect("load 2nd info from chunk fail");
                    db.table_info.push(info);
                }
                table_info_size = table_info_size - 2;
            }

            Ok(db)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn commit(&self) -> Result<u32> {
        Ok(0)
    }

    /// Update the header of Db, but not the Table
    /// The commit of Table will automatically trigger the commit of Db
    #[cfg(target_arch = "wasm32")]
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

    /// alloc storage space for table
    pub fn alloc_table_storage(&mut self, sig: TableSig, raw_length: u32) -> Result<Range<u32>> {
        let mut modify_list: Vec<(Range<u32>, Range<u32>)> = Vec::new();
        let mut info_raw_length = self.table_info.len() / 2;
        if self.table_info.len() % 2 > 0 {
            info_raw_length = info_raw_length + 1;
        }

        let mut previous_end = (info_raw_length + 1) as u32;

        let mut output: Option<Range<u32>> = None;
        let mut new_range: Option<Range<u32>> = None;
        for info in self.table_info.iter_mut() {
            if info.sig == sig {
                new_range = Some(Range {
                    start: previous_end,
                    end: previous_end + raw_length,
                });
                output = new_range.clone();
            } else if info.range.start != previous_end {
                new_range = Some(Range {
                    start: previous_end,
                    end: previous_end + info.range.end - info.range.start,
                });
            }

            if let Some(new_range) = new_range.take() {
                modify_list.push((info.range.clone(), new_range.clone()));
                info.range = new_range;
            }

            previous_end = info.range.end;
        }

        migration_table(modify_list)?;

        output.ok_or(Error::TableNotExist(format!("Table [sig: {:?}]", sig)).into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn migration_table(list: Vec<(Range<u32>, Range<u32>)>) -> Result<()> {
    Ok(())
}

/// Migrate table from Range to Range
#[cfg(target_arch = "wasm32")]
fn migration_table(mut list: Vec<(Range<u32>, Range<u32>)>) -> Result<()> {
    let mut addr: [u8; 32] = [0; 32];

    while let Some((
        Range::<u32> {
            start: _before_range_start,
            end: before_range_end,
        },
        Range::<u32> {
            start: new_range_start,
            end: new_range_end,
        },
    )) = list.pop()
    {
        for i in 1..=new_range_end - new_range_start {
            storage_index_to_addr((before_range_end - i) as usize, &mut addr);
            let buffer: [u8; 32] = storage_load(&addr.into()).bytes;
            storage_index_to_addr((new_range_end - i) as usize, &mut addr);
            storage_store(&addr.into(), &buffer.into());
        }
    }
    Ok(())
}

fn get_table_signature(table_name: &str) -> TableSig {
    let mut sig = [0; 4];
    let mut hasher = Keccak::v256();
    hasher.update(table_name.as_bytes());
    hasher.finalize(&mut sig);
    sig
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_table_storage_with_tables() {
        #[derive(Default, Serialize, Deserialize)]
        struct Person1 {
            trusted: bool,
        }
        impl Record for Person1 {}

        #[derive(Default, Serialize, Deserialize)]
        struct Person2 {
            trusted: bool,
        }
        impl Record for Person2 {}

        #[derive(Default, Serialize, Deserialize)]
        struct Person3 {
            trusted: bool,
        }
        impl Record for Person3 {}

        let mut db = Db::default();
        db.create_table::<Person1>();
        db.create_table::<Person2>();
        db.create_table::<Person3>();
        assert!(db.table_info.len() == 3);
        for i in 0..3 {
            assert!(db.table_info[i].range == Range::<u32> { start: 2, end: 2 });
        }
        // There are not record in Person1, Person3, and there 3 raw of record in Person2
        let r = db
            .alloc_table_storage(get_table_signature(std::any::type_name::<Person2>()), 3)
            .unwrap();
        assert!(db.table_info[0].range == Range::<u32> { start: 3, end: 3 });
        assert!(db.table_info[1].range == Range::<u32> { start: 3, end: 6 });
        assert!(db.table_info[2].range == Range::<u32> { start: 6, end: 6 });
    }
    #[test]
    fn test_alloc_table_storage_with_one_table() {
        #[derive(Default, Serialize, Deserialize)]
        struct Person {
            trusted: bool,
        }
        impl Record for Person {}

        let mut db = Db::default();
        db.create_table::<Person>();
        assert!(db.table_info.len() == 1);
        assert!(db.table_info[0].range == Range::<u32> { start: 2, end: 2 });
        let r = db.alloc_table_storage(get_table_signature(std::any::type_name::<Person>()), 1);

        assert!(r.unwrap() == Range::<u32> { start: 2, end: 3 });
        assert!(db.table_info[0].range == Range::<u32> { start: 2, end: 3 });
    }
}
