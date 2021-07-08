use std::marker::PhantomData;

use anyhow::Result;
#[cfg(target_arch = "wasm32")]
use ewasm_api::storage_store;

use crate::rdb::db::{Db, TableInfo};
use crate::types::{Raw, Row};
use crate::utils::storage_index_to_addr;

pub struct Table<T> {
    pub(crate) info: TableInfo,
    pub(crate) data: Vec<Row>,
    pub(crate) phantom: PhantomData<T>,
}

impl<T> Table<T> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn commit(&self) -> Result<u32> {
        Ok(0)
    }

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
