use std::{convert::TryFrom, fmt, iter::FromIterator};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_derive::Deserialize as DeserializeDerive;

use crate::types::*;

/// The small storage unit in the contract, which contains 32 bytes.
#[derive(Clone)]
pub struct Raw {
    pub(crate) bytes: [u8; 32],
    // TODO: design a feature using the flag to write only needed
    // flag: u8,
}

#[derive(DeserializeDerive, Debug, PartialEq)]
struct RawHelper {
    e01: u8,
    e02: u8,
    e03: u8,
    e04: u8,
    e05: u8,
    e06: u8,
    e07: u8,
    e08: u8,
    e09: u8,
    e10: u8,
    e11: u8,
    e12: u8,
    e13: u8,
    e14: u8,
    e15: u8,
    e16: u8,
    e17: u8,
    e18: u8,
    e19: u8,
    e20: u8,
    e21: u8,
    e22: u8,
    e23: u8,
    e24: u8,
    e25: u8,
    e26: u8,
    e27: u8,
    e28: u8,
    e29: u8,
    e30: u8,
    e31: u8,
    e32: u8,
}

impl Serialize for Raw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.bytes.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Raw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(
            |RawHelper {
                 e01,
                 e02,
                 e03,
                 e04,
                 e05,
                 e06,
                 e07,
                 e08,
                 e09,
                 e10,
                 e11,
                 e12,
                 e13,
                 e14,
                 e15,
                 e16,
                 e17,
                 e18,
                 e19,
                 e20,
                 e21,
                 e22,
                 e23,
                 e24,
                 e25,
                 e26,
                 e27,
                 e28,
                 e29,
                 e30,
                 e31,
                 e32,
             }| {
                let bytes: [u8; 32] = [
                    e01, e02, e03, e04, e05, e06, e07, e08, e09, e10, e11, e12, e13, e14, e15, e16,
                    e17, e18, e19, e20, e21, e22, e23, e24, e25, e26, e27, e28, e29, e30, e31, e32,
                ];
                Raw { bytes, /*flag: 1*/ }
            },
        )
    }
}

impl Default for Raw {
    fn default() -> Self {
        Raw {
            bytes: [0; 32],
            // flag: 0,
        }
    }
}

impl AsRef<[u8]> for Raw {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Raw {
    pub(crate) fn new(slice: &[u8]) -> Self {
        if slice.len() <= 32 {
            let mut instance = Self::default();
            instance.bytes[..slice.len()].copy_from_slice(slice);
            instance
        } else {
            panic!("input slice is bigger than a Raw");
        }
    }
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.bytes)
    }
    pub fn to_bytes32(&self) -> [u8; 32] {
        self.bytes
    }
}

impl FromIterator<u8> for Raw {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        let bs: Vec<u8> = iter.into_iter().collect();
        bs.into()
    }
}

impl From<&[u8]> for Raw {
    fn from(slice: &[u8]) -> Self {
        Raw::new(slice)
    }
}

impl From<&str> for Raw {
    fn from(s: &str) -> Self {
        Self::from(s.as_bytes())
    }
}

impl From<String> for Raw {
    fn from(s: String) -> Self {
        Self::from(s.as_bytes())
    }
}

impl From<&String> for Raw {
    fn from(s: &String) -> Self {
        Self::from(s.as_bytes())
    }
}

impl From<&Raw> for Raw {
    fn from(v: &Self) -> Self {
        v.clone()
    }
}

impl From<Vec<u8>> for Raw {
    fn from(v: Vec<u8>) -> Self {
        Raw::new(&v)
    }
}

impl From<Row> for Vec<Raw> {
    fn from(v: Row) -> Self {
        v.inner
    }
}

impl From<Box<[u8]>> for Raw {
    fn from(v: Box<[u8]>) -> Self {
        Raw::new(&v)
    }
}

impl std::borrow::Borrow<[u8]> for Raw {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

impl std::borrow::Borrow<[u8]> for &Raw {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

impl TryFrom<&Row> for Raw {
    type Error = &'static str;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        if value.len() <= 1 {
            Err("Row is bigger than raw")
        } else {
            Ok(value.inner[0].clone())
        }
    }
}

impl TryFrom<Row> for Raw {
    type Error = &'static str;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        if value.len() <= 1 {
            Err("Row is bigger than raw")
        } else {
            Ok(value.inner[0].clone())
        }
    }
}

macro_rules! from_array {
    ($($s:expr),*) => {
        $(
            impl From<&[u8; $s]> for Raw {
                fn from(v: &[u8; $s]) -> Self {
                    Self::from(&v[..])
                }
            }
        )*
    }
}

from_array!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
);

impl Ord for Raw {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl PartialOrd for Raw {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for Raw {
    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialEq<[u8]> for Raw {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref() == other
    }
}

impl Eq for Raw {}

impl fmt::Debug for Raw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}
