//! helps serialized value object into raw and also deserialize the raw back to object
use std::borrow::Borrow;
use std::convert::TryFrom;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::types::{Raw, Row};

/// helps to serialize struct as Value to row or deserialized from row
/// ```
/// | 1st bytes | ...    | padding                   |
/// |-----------|--------|---------------------------|
/// | Header    | Binary | padding to n times Byte32 |
/// ```
/// Header is the number of bytes for binary
//TODO make Header bigger for big value object storage
pub trait Value: Sized + Serialize + DeserializeOwned {
    fn to_row_value(&self) -> Result<Row> {
        let mut bin = bincode::serialize(&self).expect("serialize a value fail");
        let length = bin.len();
        let header = ((length + 1) & 31) as u8; // padding bytes
        let mut vec = vec![header];
        vec.append(&mut bin);
        Ok(vec.into())
    }
    fn from_row_value(r: &Row) -> Result<Self> {
        let buffer: &[u8] = r.borrow();
        let header = buffer[0] as usize;
        let instance: Self = bincode::deserialize(&buffer[1..buffer.len() - header + 1])
            .expect("load binary to row fail fail");
        Ok(instance)
    }
}

impl Value for Raw {
    fn to_row_value(&self) -> Result<Row> {
        Ok(self.into())
    }

    fn from_row_value(r: &Row) -> Result<Self> {
        Ok(Raw::try_from(r).expect("Data loose from Row to Raw"))
    }
}

impl Value for Row {
    fn to_row_value(&self) -> Result<Row> {
        Ok(self.clone())
    }

    fn from_row_value(r: &Row) -> Result<Self> {
        Ok(r.clone())
    }
}
