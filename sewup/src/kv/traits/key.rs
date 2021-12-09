//! helps serialized key object into raw and also deserialize the raw back to object
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};

use anyhow::Result;
use cryptoxide::blake2s::Blake2s;
use cryptoxide::mac::Mac;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::types::{Address, Raw, Row};

/// helps to serialize struct as Key to row or deserialized from row
/// ```compile_fail
/// | 1st bytes | ...    | padding                   |
/// |-----------|--------|---------------------------|
/// | Header    | Binary | padding to n times Byte32 |
/// ```
/// Header is the number of bytes for binary
pub trait Key: Clone + Sized + Serialize + DeserializeOwned {
    fn from_row_key(r: &Row) -> Result<Self> {
        let buffer: &[u8] = r.borrow();
        let header = buffer[0] as usize;
        let instance: Self = bincode::deserialize(&buffer[1..buffer.len() - header])
            .expect("load binary to key fail");
        Ok(instance)
    }

    fn to_row_key(&self) -> Result<Row> {
        let mut bin = bincode::serialize(&self).expect("serialize a key fail");
        let length = bin.len();
        let header = ((length + 1) & 31) as u8; // padding bytes
        let mut vec = vec![header];
        vec.append(&mut bin);
        Ok(vec.into())
    }

    fn gen_hash_key(&self, key_len: u32, value_len: u32) -> Result<Raw> {
        let mut bytes: [u8; 32] = [0; 32];
        let bin = bincode::serialize(&self).expect("serialize a key fail");

        let mut b = Blake2s::new(24);
        b.input(&bin);
        b.raw_result(&mut bytes[0..24]);

        let mut length_buffer = key_len.to_be_bytes();
        length_buffer.swap_with_slice(&mut bytes[24..28]);
        length_buffer = (value_len as u32).to_be_bytes();
        length_buffer.swap_with_slice(&mut bytes[28..32]);

        Ok(Raw::from(&bytes))
    }

    fn gen_hash(&self) -> Result<[u8; 24]> {
        let mut hash: [u8; 24] = [0; 24];
        let bin = bincode::serialize(&self).expect("serialize a key fail");

        let mut b = Blake2s::new(24);
        b.input(&bin);
        b.raw_result(&mut hash);
        Ok(hash)
    }
}

pub trait AsHashKey {
    fn get_size_from_hash(&self, hash: &[u8; 24]) -> (bool, u32, u32);
    fn get_size(&self) -> (u32, u32);
}

impl AsHashKey for Raw {
    fn get_size_from_hash(&self, hash: &[u8; 24]) -> (bool, u32, u32) {
        let (k_size, v_size) = self.get_size();
        (*hash == self.bytes[0..24], k_size, v_size)
    }
    fn get_size(&self) -> (u32, u32) {
        let key_bytes: &[u8; 4] = (&self.bytes[24..28]).try_into().unwrap();
        let value_bytes: &[u8; 4] = (&self.bytes[28..32]).try_into().unwrap();
        (
            u32::from_be_bytes(*key_bytes),
            u32::from_be_bytes(*value_bytes),
        )
    }
}

impl Key for Raw {
    fn from_row_key(x: &Row) -> Result<Self> {
        Ok(Raw::try_from(x).expect("Data loose from Row to Raw"))
    }
    fn to_row_key(&self) -> Result<Row> {
        Ok(self.into())
    }
}

impl Key for Row {
    fn from_row_key(x: &Row) -> Result<Self> {
        Ok(x.clone())
    }
}

impl Key for Address {
    fn from_row_key(_x: &Row) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let addr = Address {
            ..Default::default()
        };
        #[cfg(target_arch = "wasm32")]
        let addr: Address = {
            let raw: Raw = _x.try_into().expect("row key should at least one raw");
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
    fn to_row_key(&self) -> Result<Row> {
        #[cfg(not(target_arch = "wasm32"))]
        let raw: Raw = Raw::default();

        #[cfg(target_arch = "wasm32")]
        let raw: Raw = self.into();

        Ok(raw.into())
    }
}

macro_rules! primitive_key {
    ( $($t:ty),* ) => {
        $(
            impl Key for $t{
                fn from_row_key(x: &Row) -> Result<Self> {
                    let r: Raw = TryFrom::try_from(x).expect("primitive key should be 1 Raw");
                    Ok(r.into())
                }
                fn to_row_key(&self) -> Result<Row> {
                    let r = Raw::from(*self);
                    Ok(r.into())
                }
            }
         )*
    }
}

primitive_key!(u8, u16, u32, u64, usize);

impl Key for String {
    fn from_row_key(x: &Row) -> Result<Self> {
        Ok(x.to_utf8_string()?)
    }
    fn to_row_key(&self) -> Result<Row> {
        Ok(self.into())
    }
}

macro_rules! sized_string_key {
    ( $($n:expr),* ) => {
        $(
            impl Key for [Raw; $n] {
                fn from_row_key(r: &Row) -> Result<Self> {
                    let mut buffer: [Raw; $n] = Default::default();
                    buffer.copy_from_slice(&r.inner[0..$n]);
                    Ok(buffer)
                }
                fn to_row_key(&self) -> Result<Row> {
                    Ok(self.to_vec().into())
                }
            }
         )*
    }
}

sized_string_key!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
);
