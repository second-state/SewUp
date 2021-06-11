use std::iter::FromIterator;

use serde_derive::{Deserialize, Serialize};

use crate::types::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Row {
    pub(super) inner: Vec<Raw>,
}

impl Row {
    pub(crate) fn new(slice: &[u8]) -> Self {
        let inner: Vec<Raw> = slice
            .into_iter()
            .map(|b| *b)
            .collect::<Vec<u8>>()
            .chunks(32)
            .fold(Vec::<Raw>::new(), |mut vec, chunk| {
                vec.push(chunk.into());
                vec
            });
        Self { inner }
    }
}

impl Default for Row {
    fn default() -> Self {
        Row { inner: Vec::new() }
    }
}

impl ExactSizeIterator for Row {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl From<Vec<Raw>> for Row {
    fn from(v: Vec<Raw>) -> Self {
        Self { inner: v }
    }
}

impl Iterator for Row {
    type Item = u8;

    #[allow(clippy::never_loop)]
    fn next(&mut self) -> Option<Self::Item> {
        for raw in self.inner.iter() {
            for b in raw.bytes.iter() {
                return Some(*b);
            }
        }
        None
    }
}

impl FromIterator<u8> for Row {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        let row: Vec<Raw> = iter.into_iter().collect::<Vec<u8>>().chunks(32).fold(
            Vec::<Raw>::new(),
            |mut vec, chunk| {
                vec.push(chunk.into());
                vec
            },
        );
        row.into()
    }
}

impl From<&[u8]> for Row {
    fn from(slice: &[u8]) -> Self {
        Row::new(slice)
    }
}

impl From<&str> for Row {
    fn from(s: &str) -> Self {
        Self::from(s.as_bytes())
    }
}

impl From<String> for Row {
    fn from(s: String) -> Self {
        Self::from(s.as_bytes())
    }
}

impl From<&String> for Row {
    fn from(s: &String) -> Self {
        Self::from(s.as_bytes())
    }
}

impl From<&Raw> for Row {
    fn from(v: &Raw) -> Self {
        Self {
            inner: vec![v.clone()],
        }
    }
}

impl From<Raw> for Row {
    fn from(v: Raw) -> Self {
        Self { inner: vec![v] }
    }
}

impl From<Vec<u8>> for Row {
    fn from(v: Vec<u8>) -> Self {
        let inner: Vec<Raw> = v.into_iter().collect::<Vec<u8>>().chunks(32).fold(
            Vec::<Raw>::new(),
            |mut vec, chunk| {
                vec.push(chunk.into());
                vec
            },
        );
        Self { inner }
    }
}

impl From<Box<[u8]>> for Row {
    fn from(v: Box<[u8]>) -> Self {
        Row::new(&v)
    }
}
