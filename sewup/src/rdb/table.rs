use std::marker::PhantomData;

use anyhow::Result;

use crate::rdb::db::{Db, TableInfo};
use crate::types::Row;

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
    pub fn commit(&self) -> Result<u32> {
        // Currently, wasm runs in single thread mode, so it is ok to do this.
        let mut db = Db::load(None)?;

        Ok(0)
    }
}
