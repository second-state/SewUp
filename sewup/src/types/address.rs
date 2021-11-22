use std::convert::TryInto;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(target_arch = "wasm32")]
pub use ewasm_api::types::{Address as EwasmAddress, Bytes20};

#[cfg(target_arch = "wasm32")]
use crate::types::Raw;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, PartialEq, Default, Debug)]
pub struct AddressType {
    pub inner: [u8; 20],
}

#[cfg(not(target_arch = "wasm32"))]
impl Serialize for AddressType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes32: [u8; 32] = [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            self.inner[0],
            self.inner[1],
            self.inner[2],
            self.inner[3],
            self.inner[4],
            self.inner[5],
            self.inner[6],
            self.inner[7],
            self.inner[8],
            self.inner[9],
            self.inner[10],
            self.inner[11],
            self.inner[12],
            self.inner[13],
            self.inner[14],
            self.inner[15],
            self.inner[16],
            self.inner[17],
            self.inner[18],
            self.inner[19],
        ];
        bytes32.serialize(serializer)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<'de> Deserialize<'de> for AddressType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|bytes: [u8; 32]| AddressType {
            inner: [
                bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17], bytes[18],
                bytes[19], bytes[20], bytes[21], bytes[22], bytes[23], bytes[24], bytes[25],
                bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
            ],
        })
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
pub struct AddressType {
    pub(crate) inner: EwasmAddress,
}

#[cfg(target_arch = "wasm32")]
impl Default for AddressType {
    fn default() -> Self {
        Self::from_str("0x0000000000000000000000000000000000000000").unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Debug for AddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.inner.bytes;
        f.debug_struct("Address").field("inner", &bytes).finish()
    }
}

/// Address is a 20 bytes binary, you can build a Address with hex string easily.
/// ```
/// let address = sewup::types::Address::from_str("8663DBF0cC68AaF37fC8BA262F2df4c666a41993").unwrap();
///
/// let same_address = sewup::types::Address::from_str("0x8663DBF0cC68AaF37fC8BA262F2df4c666a41993").unwrap();
///
/// assert!(address == same_address);
/// ```
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
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        let hex_s: &str = if s.starts_with("0x") {
            &s[2..s.len()]
        } else {
            s
        };
        Ok(Self {
            inner: hex::decode(hex_s)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("hex str can not convert to [u8; 20]"))?,
        })
    }
}
