use std::borrow::Borrow;
use std::convert::TryFrom;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Raw, Row};

pub trait Value<'a>: Sized + Serialize + Deserialize<'a> {
    fn to_raw_value(&self) -> Result<Row> {
        let mut bin = bincode::serialize(&self).expect("serialize a value fail");
        let length = bin.len();
        let header = ((length + 1) & 31) as u8; // padding bytes
        let mut vec = vec![header];
        vec.append(&mut bin);
        Ok(vec.into())
    }
    fn from_raw_value(r: &'a Row) -> Result<Self> {
        let buffer: &[u8] = r.borrow();
        let header = buffer[0] as usize;
        let instance: Self = bincode::deserialize(&buffer[1..buffer.len() - header + 1])
            .expect("load db binary fail");
        Ok(instance)
    }
}

impl<'a> Value<'a> for Raw {
    fn to_raw_value(&self) -> Result<Row> {
        Ok(self.into())
    }

    fn from_raw_value(r: &'a Row) -> Result<Self> {
        Ok(Raw::try_from(r).expect("Data loose from Row to Raw"))
    }
}

impl<'a> Value<'a> for Row {
    fn to_raw_value(&self) -> Result<Row> {
        Ok(self.clone())
    }

    fn from_raw_value(r: &'a Row) -> Result<Self> {
        Ok(r.clone())
    }
}

// impl Value for Vec<u8> {
//     fn to_raw_value(&self) -> Result<Raw> {
//         Ok(self.as_slice().into())
//     }

//     fn from_raw_value(r: Raw) -> Result<Self> {
//         Ok(r.as_ref().to_vec())
//     }
// }

// impl Value for String {
//     fn to_raw_value(&self) -> Result<Raw> {
//         Ok(self.as_str().into())
//     }

//     fn from_raw_value(r: Raw) -> Result<Self> {
//         let x = r.as_ref().to_vec();
//         Ok(String::from_utf8(x)?)
//     }
// }
