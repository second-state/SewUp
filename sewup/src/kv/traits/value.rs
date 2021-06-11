use std::convert::TryFrom;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Raw, Row};

pub trait Value<'a>: Sized + Serialize + Deserialize<'a> {
    fn to_raw_value(&self) -> Result<Row>;
    fn from_raw_value(r: Row, padding: u8) -> Result<Self>;
}

impl Value<'_> for Raw {
    fn to_raw_value(&self) -> Result<Row> {
        Ok(self.into())
    }

    fn from_raw_value(r: Row, _padding: u8) -> Result<Self> {
        Ok(Raw::try_from(r).expect("Data loose from Row to Raw"))
    }
}

impl Value<'_> for Row {
    fn to_raw_value(&self) -> Result<Row> {
        Ok(self.clone())
    }

    fn from_raw_value(r: Row, _padding: u8) -> Result<Self> {
        Ok(r)
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
