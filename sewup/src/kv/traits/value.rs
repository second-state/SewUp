use anyhow::Result;

use crate::types::Raw;

/// convert between types and `Raw`
pub trait Value: Sized {
    /// Wrapper around AsRef<[u8]>
    fn to_raw_value(&self) -> Result<Raw>;

    /// Convert from Raw
    fn from_raw_value(r: Raw) -> Result<Self>;
}

impl Value for Raw {
    fn to_raw_value(&self) -> Result<Raw> {
        Ok(self.clone())
    }

    fn from_raw_value(r: Raw) -> Result<Self> {
        Ok(r)
    }
}

impl Value for std::sync::Arc<[u8]> {
    fn to_raw_value(&self) -> Result<Raw> {
        Ok(self.as_ref().into())
    }

    fn from_raw_value(r: Raw) -> Result<Self> {
        Ok(r.as_ref().into())
    }
}

impl Value for Vec<u8> {
    fn to_raw_value(&self) -> Result<Raw> {
        Ok(self.as_slice().into())
    }

    fn from_raw_value(r: Raw) -> Result<Self> {
        Ok(r.as_ref().to_vec())
    }
}

impl Value for String {
    fn to_raw_value(&self) -> Result<Raw> {
        Ok(self.as_str().into())
    }

    fn from_raw_value(r: Raw) -> Result<Self> {
        let x = r.as_ref().to_vec();
        Ok(String::from_utf8(x)?)
    }
}
