use std::borrow::Borrow;
use std::convert::TryFrom;

use anyhow::Result;
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

// impl<'a> Key<'a> for &'a [u8] {
//     fn from_raw_key(x: &'a Raw) -> Result<&'a [u8]> {
//         Ok(x.as_ref())
//     }
// }

// impl<'a> Key<'a> for &'a str {
//     fn from_raw_key(x: &'a Raw) -> Result<Self> {
//         Ok(std::str::from_utf8(x.as_ref())?)
//     }
// }

// impl<'a> Key<'a> for Vec<u8> {
//     fn from_raw_key(r: &Raw) -> Result<Self> {
//         Ok(r.as_ref().to_vec())
//     }
// }

// impl<'a> Key<'a> for String {
//     fn from_raw_key(x: &Raw) -> Result<Self> {
//         Ok(std::str::from_utf8(x.as_ref())?.to_string())
//     }
// }
