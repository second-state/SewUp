//! helps serialized value object into raw and also deserialize the raw back to object
use std::borrow::Borrow;
use std::convert::TryFrom;
#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::types::{Address, Raw, Row};

/// helps to serialize struct as Value to row or deserialized from row
/// ```compile_fail
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

impl Value for Address {
    fn from_row_value(x: &Row) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let addr = Address {};
        #[cfg(target_arch = "wasm32")]
        let addr: Address = {
            let raw: Raw = x.try_into().expect("row key should at least one raw");
            let byte20: [u8; 20] = [
                raw.bytes[12],
                raw.bytes[13],
                raw.bytes[14],
                raw.bytes[15],
                raw.bytes[16],
                raw.bytes[17],
                raw.bytes[18],
                raw.bytes[19],
                raw.bytes[20],
                raw.bytes[21],
                raw.bytes[22],
                raw.bytes[23],
                raw.bytes[24],
                raw.bytes[25],
                raw.bytes[26],
                raw.bytes[27],
                raw.bytes[28],
                raw.bytes[29],
                raw.bytes[30],
                raw.bytes[31],
            ];
            Address {
                inner: ewasm_api::types::Address::from(byte20),
            }
        };
        Ok(addr)
    }
    fn to_row_value(&self) -> Result<Row> {
        #[cfg(not(target_arch = "wasm32"))]
        let raw: Raw = Raw::default();

        #[cfg(target_arch = "wasm32")]
        let raw: Raw = self.into();

        Ok(raw.into())
    }
}
