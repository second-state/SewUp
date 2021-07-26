use std::marker::PhantomData;

use anyhow::Result;
#[cfg(target_arch = "wasm32")]
use ewasm_api::{storage_load, storage_store};

#[cfg(target_arch = "wasm32")]
use crate::rdb::db::Db;
use crate::rdb::db::TableInfo;
use crate::rdb::{
    errors::Error,
    traits::{Record, HEADER_SIZE},
};
#[cfg(target_arch = "wasm32")]
use crate::types::Raw;
use crate::types::Row;
#[cfg(target_arch = "wasm32")]
use crate::utils::storage_index_to_addr;

pub struct Table<T: Record> {
    pub(crate) info: TableInfo,
    pub(crate) data: Vec<Row>,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: Record> Table<T> {
    /// Add a new record into table
    pub fn add_record(&mut self, instance: T) -> Result<usize> {
        self.data.push(instance.to_row(self.info.record_raw_size)?);
        Ok(self.data.len())
    }

    /// Get a record with specific id
    pub fn get_record(&self, id: usize) -> Result<T> {
        return if id == 0 {
            Err(Error::RecordIdCorrect.into())
        } else if self.data.len() == 0 {
            Err(Error::TableIsEmpty.into())
        } else {
            let mut row: Row = self.data[id - 1].clone();
            row.make_buffer();
            if let Some(i) = T::from_row(&row) {
                Ok(i)
            } else {
                Err(Error::RecordDeleted.into())
            }
        };
    }

    /// Get all records
    pub fn all_records(&self) -> Result<Vec<T>> {
        let mut output: Vec<T> = Vec::new();
        for r in self.data.iter() {
            let mut buffer_row = r.clone();
            buffer_row.make_buffer();
            if let Some(i) = T::from_row(&buffer_row) {
                output.push(i);
            }
        }
        Ok(output)
    }

    #[allow(bare_trait_objects)]
    /// Filter the records
    pub fn filter_records(&self, filter: &Fn(&T) -> bool) -> Result<Vec<T>> {
        let mut output: Vec<T> = Vec::new();
        for r in self.data.iter() {
            let mut buffer_row = r.clone();
            buffer_row.make_buffer();
            if let Some(i) = T::from_row(&buffer_row) {
                if filter(&i) {
                    output.push(i);
                }
            }
        }
        Ok(output)
    }

    /// Update or delete a record with specific id
    pub fn update_record(&mut self, id: usize, instance: Option<T>) -> Result<()> {
        return if id == 0 {
            Err(Error::RecordIdCorrect.into())
        } else if self.data.len() == 0 {
            Err(Error::TableIsEmpty.into())
        } else {
            if let Some(instance) = instance {
                self.data[id - 1] = instance.to_row(self.info.record_raw_size)?;
            } else {
                self.data[id - 1].wipe_header(HEADER_SIZE as usize)
            }
            Ok(())
        };
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn load_data(self) -> Result<Self> {
        Ok(self)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn load_data(self) -> Result<Self> {
        let Self { info, phantom, .. } = self;
        let mut db = Db::load(None)?;
        let mut addr: [u8; 32] = [0; 32];
        let mut data: Vec<Row> = Vec::new();
        let mut buffer: Vec<Raw> = Vec::new();

        for storage_idx in info.clone().range {
            storage_index_to_addr(storage_idx as usize, &mut addr);
            let raw: Raw = (&storage_load(&addr.into()).bytes).into();
            buffer.push(raw);
            if buffer.len() % info.record_raw_size as usize == 0 {
                data.push(buffer.into());
                buffer = Vec::new();
            }
        }

        Ok(Table::<T> {
            info,
            data,
            phantom,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn commit(&self) -> Result<u32> {
        Ok(0)
    }

    /// Dump the data of table on chain, and also update all the table info
    #[cfg(target_arch = "wasm32")]
    pub fn commit(mut self) -> Result<u32> {
        // Currently, wasm runs in single thread mode, so it is ok to do this.
        // If multiple treading happened, use Arc on DB and refactor this
        let raw_length = self.data.iter().fold(0u32, |sum, r| sum + r.len() as u32);
        let mut db = Db::load(None)?;
        let mut addr: [u8; 32] = [0; 32];

        let mut raw_list: Vec<Raw> = Vec::new();
        for row in self.data.drain(..) {
            raw_list.append(&mut row.into_raw_vec());
        }

        for (idx, storage_idx) in db
            .alloc_table_storage(self.info.sig, raw_length)?
            .enumerate()
        {
            storage_index_to_addr(storage_idx as usize, &mut addr);
            storage_store(&addr.into(), &raw_list[idx].to_bytes32().into());
        }

        db.commit()?;

        Ok(raw_list.len() as u32)
    }
}
