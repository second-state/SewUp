#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(not(target_arch = "wasm32"))]
use serde_derive::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
pub use ewasm_api::types::{Address as EwasmAddress, Bytes20};

#[cfg(target_arch = "wasm32")]
use crate::types::Raw;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Deserialize, Serialize)]
pub struct AddressType {}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct AddressType {
    pub(crate) inner: EwasmAddress,
}

pub type Address = AddressType;

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
