//! traits for Record, which the item in an table
use std::borrow::Borrow;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::rdb::errors::Error;
use crate::types::{Raw, Row};

//TODO make Header bigger for big object storage
pub const HEADER_SIZE: u32 = 1;

/// helps to serialize struct to row or deserialized from row
/// ```
/// | 1st bytes | ...    | padding                   |
/// |-----------|--------|---------------------------|
/// | Header    | Binary | padding to n times Byte32 |
/// ```
/// Header is the number of bytes for binary,
/// Record can be delete by simple mark the header zero
pub trait Record: Sized + Serialize + DeserializeOwned {
    fn from_row(r: &Row) -> Option<Self> {
        let buffer: &[u8] = r.borrow();
        let header = buffer[0] as usize;
        return if header == 0 {
            None
        } else {
            let instance: Self =
                bincode::deserialize(&buffer[1..buffer.len() - header + 1]).expect("load a record");
            Some(instance)
        };
    }

    fn to_row(&self, row_length: u32) -> Result<Row> {
        let mut bin = bincode::serialize(&self).expect("serialize a record fail");
        let length = bin.len();
        let header = ((length + 1) & 31) as u8; // padding bytes
        let mut vec = vec![header];
        vec.append(&mut bin);
        let row: Row = vec.into();

        return if row.len() as u32 == row_length {
            Ok(row)
        } else {
            Err(Error::RecordNotSized.into())
        };
    }
}
