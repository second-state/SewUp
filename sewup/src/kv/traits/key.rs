use anyhow::Result;

use crate::types::Raw;

pub trait Key<'a>: Sized + AsRef<[u8]> {
    fn from_raw_key(r: &'a Raw) -> Result<Self>;

    fn to_raw_key(&self) -> Result<Raw> {
        Ok(self.as_ref().into())
    }
}

impl<'a> Key<'a> for Raw {
    fn from_raw_key(x: &Raw) -> Result<Self> {
        Ok(x.clone())
    }
}

impl<'a> Key<'a> for &'a [u8] {
    fn from_raw_key(x: &'a Raw) -> Result<&'a [u8]> {
        Ok(x.as_ref())
    }
}

impl<'a> Key<'a> for &'a str {
    fn from_raw_key(x: &'a Raw) -> Result<Self> {
        Ok(std::str::from_utf8(x.as_ref())?)
    }
}

impl<'a> Key<'a> for Vec<u8> {
    fn from_raw_key(r: &Raw) -> Result<Self> {
        Ok(r.as_ref().to_vec())
    }
}

impl<'a> Key<'a> for String {
    fn from_raw_key(x: &Raw) -> Result<Self> {
        Ok(std::str::from_utf8(x.as_ref())?.to_string())
    }
}
