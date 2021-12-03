use std::{
    convert::{TryFrom, TryInto},
    fmt,
    iter::FromIterator,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_derive::Deserialize as DeserializeDerive;

use crate::types::Row;

#[cfg(target_arch = "wasm32")]
use crate::types::Address as SewupAddress;
#[cfg(target_arch = "wasm32")]
use ewasm_api::types::Address;

/// The storage unit in the contract of Ethereum, which contains 32 bytes
/// The structure is easiler to try_from `Row`; from `Address`, bytes, unsigned integers;
/// or into `bytes32`, `bytes20`, etc.
#[derive(Clone, Copy, Default)]
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

    /// build Raw from the address type(bytes20) of ehtereum
    pub fn from_raw_address(addr: &[u8; 20]) -> Self {
        Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, addr[0], addr[1], addr[2],
            addr[3], addr[4], addr[5], addr[6], addr[7], addr[8], addr[9], addr[10], addr[11],
            addr[12], addr[13], addr[14], addr[15], addr[16], addr[17], addr[18], addr[19],
        ])
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.bytes)
    }

    /// return the storage unit of Ethereum
    pub fn to_bytes32(&self) -> [u8; 32] {
        self.bytes
    }

    pub fn to_bytes20(&self) -> [u8; 20] {
        self.bytes[12..32].try_into().unwrap()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    /// wipe the header with header size in bytes
    pub fn wipe_header(&mut self, header_size: usize) {
        assert!(header_size <= 32);
        for i in 0..header_size {
            self.bytes[i] = 0;
        }
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

impl From<u8> for Raw {
    fn from(num: u8) -> Self {
        Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, num,
        ])
    }
}

impl From<Raw> for u8 {
    fn from(r: Raw) -> u8 {
        r.bytes[31]
    }
}

impl From<u16> for Raw {
    fn from(num: u16) -> Self {
        let bytes = num.to_be_bytes();
        Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, bytes[0], bytes[1],
        ])
    }
}

impl From<Raw> for u16 {
    fn from(r: Raw) -> u16 {
        u16::from_be_bytes(r.bytes[30..32].try_into().unwrap())
    }
}

impl From<u32> for Raw {
    fn from(num: u32) -> Self {
        let bytes = num.to_be_bytes();
        Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, bytes[0], bytes[1], bytes[2],
            bytes[3],
        ])
    }
}

impl From<Raw> for u32 {
    fn from(r: Raw) -> u32 {
        u32::from_be_bytes(r.bytes[28..32].try_into().unwrap())
    }
}

impl From<u64> for Raw {
    fn from(num: u64) -> Self {
        let bytes = num.to_be_bytes();
        Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
            bytes[5], bytes[6], bytes[7],
        ])
    }
}

impl From<Raw> for u64 {
    fn from(r: Raw) -> u64 {
        u64::from_be_bytes(r.bytes[24..32].try_into().unwrap())
    }
}

impl From<usize> for Raw {
    fn from(num: usize) -> Self {
        let bytes = num.to_be_bytes();
        #[cfg(target_pointer_width = "64")]
        return Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
            bytes[5], bytes[6], bytes[7],
        ]);

        #[cfg(target_pointer_width = "32")]
        return Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, bytes[0], bytes[1], bytes[2],
            bytes[3],
        ]);
    }
}

impl From<Raw> for usize {
    fn from(r: Raw) -> usize {
        #[cfg(target_pointer_width = "64")]
        return u64::from_be_bytes(r.bytes[24..32].try_into().unwrap()) as usize;

        #[cfg(target_pointer_width = "32")]
        return u32::from_be_bytes(r.bytes[28..32].try_into().unwrap()) as usize;
    }
}

macro_rules! signed_int_convert {
    ($($n:expr),*) => {
        paste::paste! {
            $(
                impl From< [<i $n>] > for Raw {
                    fn from(num: [<i $n>]) -> Self {
                        let n = unsafe { std::mem::transmute::<[<i $n>], [<u $n>]>(num) };
                        n.into()
                    }
                }

                impl From<Raw> for [<i $n>] {
                    fn from(r: Raw) -> [<i $n>] {
                        let num = [<u $n>]::from(r);
                        unsafe { std::mem::transmute::<[<u $n>], [<i $n>]>(num) }
                    }
                }
            )*
        }
    }
}

signed_int_convert!(8, 16, 32, 64, size);

#[cfg(target_arch = "wasm32")]
impl From<Address> for Raw {
    fn from(addr: Address) -> Self {
        let bytes = addr.bytes;
        Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, bytes[0], bytes[1],
            bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17],
            bytes[18], bytes[19],
        ])
    }
}

#[cfg(target_arch = "wasm32")]
impl From<SewupAddress> for Raw {
    fn from(addr: SewupAddress) -> Self {
        addr.inner.into()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<&Address> for Raw {
    fn from(addr: &Address) -> Self {
        let bytes = addr.bytes;
        Raw::from(&[
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, bytes[0], bytes[1],
            bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17],
            bytes[18], bytes[19],
        ])
    }
}

#[cfg(target_arch = "wasm32")]
impl From<&SewupAddress> for Raw {
    fn from(addr: &SewupAddress) -> Self {
        addr.inner.into()
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
        *v
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
        if value.len() < 1 {
            Err("Row should be bigger than raw")
        } else {
            Ok(value.inner[0])
        }
    }
}

impl TryFrom<Row> for Raw {
    type Error = &'static str;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        if value.len() < 1 {
            Err("Row should be bigger than raw")
        } else {
            Ok(value.inner[0])
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
