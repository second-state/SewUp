use std::convert::TryFrom;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Raw, Row};

// TODO here
pub trait Key<'a>: Sized + Serialize + Deserialize<'a> {
    // TODO: handle half Raw for numbers to save storage
    fn from_raw_key(r: &'a Row, padding: u8) -> Result<Self>;

    fn to_raw_key(&self) -> Result<Row> {
        let _bin = bincode::serialize(&self).expect("serialize a key fail");
        // Ok(self.iter(|s| s.into()).collect::<Raw>().into())
        Ok(Row::default())
    }
}

impl<'a> Key<'a> for Raw {
    fn from_raw_key(x: &Row, _padding: u8) -> Result<Self> {
        // TODO: think twice about the assert
        // assert inner size == 1
        Ok(Raw::try_from(x).expect("Data loose from Row to Raw"))
    }
    fn to_raw_key(&self) -> Result<Row> {
        Ok(self.into())
    }
}

impl<'a> Key<'a> for Row {
    fn from_raw_key(x: &Row, _padding: u8) -> Result<Self> {
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
