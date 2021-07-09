use std::borrow::Borrow;
use std::convert::TryFrom;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::types::{Raw, Row};

//TODO make Header bigger for big object storage
pub trait Value: Sized + Serialize + DeserializeOwned {
    // XXX: typo raw
    fn to_raw_value(&self) -> Result<Row> {
        let mut bin = bincode::serialize(&self).expect("serialize a value fail");
        let length = bin.len();
        let header = ((length + 1) & 31) as u8; // padding bytes
        let mut vec = vec![header];
        vec.append(&mut bin);
        Ok(vec.into())
    }
    fn from_raw_value(r: &Row) -> Result<Self> {
        let buffer: &[u8] = r.borrow();
        let header = buffer[0] as usize;
        // XXX: fix expect msg
        let instance: Self = bincode::deserialize(&buffer[1..buffer.len() - header + 1])
            .expect("load db binary fail");
        Ok(instance)
    }
}

impl Value for Raw {
    // XXX: typo raw
    fn to_raw_value(&self) -> Result<Row> {
        Ok(self.into())
    }

    fn from_raw_value(r: &Row) -> Result<Self> {
        Ok(Raw::try_from(r).expect("Data loose from Row to Raw"))
    }
}

impl Value for Row {
    // XXX: typo raw
    fn to_raw_value(&self) -> Result<Row> {
        Ok(self.clone())
    }

    fn from_raw_value(r: &Row) -> Result<Self> {
        Ok(r.clone())
    }
}
