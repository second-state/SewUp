use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};

use anyhow::Result;
use crypto::blake2s::Blake2s;
use crypto::mac::Mac;
use serde::{Deserialize, Serialize};

use crate::types::{Raw, Row};

pub trait Key<'a>: Sized + Serialize + Deserialize<'a> {
    fn from_raw_key(r: &'a Row) -> Result<Self> {
        let buffer: &[u8] = r.borrow();
        let header = buffer[0] as usize;
        let instance: Self = bincode::deserialize(&buffer[1..buffer.len() - header + 1])
            .expect("load db binary fail");
        Ok(instance)
    }

    fn to_raw_key(&self) -> Result<Row> {
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
    fn get_size_from_hash(&self, hash: [u8; 24]) -> (bool, u32, u32);
}

impl AsHashKey for Raw {
    fn get_size_from_hash(&self, hash: [u8; 24]) -> (bool, u32, u32) {
        let key_bytes: &[u8; 4] = (&self.bytes[24..28]).try_into().unwrap();
        let value_bytes: &[u8; 4] = (&self.bytes[28..32]).try_into().unwrap();
        (
            hash == self.bytes[0..24],
            u32::from_be_bytes(*key_bytes),
            u32::from_be_bytes(*value_bytes),
        )
    }
}

impl<'a> Key<'a> for Raw {
    fn from_raw_key(x: &Row) -> Result<Self> {
        Ok(Raw::try_from(x).expect("Data loose from Row to Raw"))
    }
    fn to_raw_key(&self) -> Result<Row> {
        Ok(self.into())
    }
}

impl<'a> Key<'a> for Row {
    fn from_raw_key(x: &'a Row) -> Result<Self> {
        Ok(x.clone())
    }
}
