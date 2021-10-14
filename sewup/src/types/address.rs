#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(not(target_arch = "wasm32"))]
use serde_derive::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
pub use ewasm_api::types::{Address as EwasmAddress, Bytes20};

#[cfg(target_arch = "wasm32")]
use crate::types::Raw;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct AddressType {}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
pub struct AddressType {
    pub(crate) inner: EwasmAddress,
}

pub type Address = AddressType;

#[cfg(target_arch = "wasm32")]
impl From<EwasmAddress> for AddressType {
    fn from(inner: EwasmAddress) -> Self {
        Self { inner }
    }
}

#[cfg(target_arch = "wasm32")]
impl Serialize for AddressType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw: Raw = self.inner.into();
        raw.serialize(serializer)
    }
}

#[cfg(target_arch = "wasm32")]
impl<'de> Deserialize<'de> for AddressType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|raw: Raw| {
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
            AddressType {
                inner: ewasm_api::types::Address::from(byte20),
            }
        })
    }
}

impl AddressType {
    #[cfg(target_arch = "wasm32")]
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        let hex_s: &str = if s.starts_with("0x") {
            &s[2..s.len()]
        } else {
            s
        };
        let byte20: [u8; 20] = hex::decode(hex_s)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("hex str can not convert to [u8; 20]"))?;
        Ok(Self {
            inner: ewasm_api::types::Address::from(byte20),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_str(_: &str) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
