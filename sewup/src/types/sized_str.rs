use serde_derive::{Deserialize, Serialize};

use anyhow::Result;

use crate::types::*;

/// SizedString is a type help you store string in with a predefined size
/// ```
/// let ss = sewup::types::sized_str::SizedString::new(10).from_str("hello").unwrap();
/// assert!(ss.len() == 5);
/// assert!(ss.capacity() == 10);
/// assert_eq!(ss.to_utf8_string().unwrap(), "hello");
/// ```
#[derive(Clone, Serialize, Deserialize)]
pub struct SizedString {
    /// The size of bytes limited
    pub capacity: usize,
    pub len: usize,
    inner: Vec<Raw>,
}

impl SizedString {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            len: 0,
            inner: Vec::<Raw>::with_capacity(capacity / 32usize + 1),
        }
    }
    pub fn from_bytes(mut self, slice: &[u8]) -> Result<Self> {
        if slice.len() > self.capacity {
            return Err(errors::TypeError::SizeExcess(self.capacity).into());
        }
        self.len = slice.len();
        self.inner = slice.iter().copied().collect::<Vec<u8>>().chunks(32).fold(
            Vec::<Raw>::new(),
            |mut vec, chunk| {
                vec.push(chunk.into());
                vec
            },
        );
        for _ in 0..(self.capacity / 32usize) - (self.len / 32usize) {
            self.inner.push(Raw::default());
        }
        Ok(self)
    }

    pub fn from_str(self, s: &str) -> Result<Self> {
        self.from_bytes(s.as_bytes())
    }

    pub fn to_utf8_string(&self) -> Result<String> {
        let buf = self.inner.iter().fold(Vec::<u8>::new(), |mut vec, raw| {
            vec.extend_from_slice(raw.as_ref());
            vec
        });
        String::from_utf8(buf[0..self.len].to_vec()).map_err(|e| e.into())
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

macro_rules! into_slice {
    ($($s:expr),*) => {
        $(
            impl From<SizedString> for [Raw; $s] {
                fn from(s: SizedString) -> [Raw; $s] {
                    let mut output = Default::default();
                    <[Raw; $s] as AsMut<[Raw]>>::as_mut(&mut output).copy_from_slice(s.inner.as_slice());
                    output
                }
            }
        )*
    }
}

into_slice!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
);
