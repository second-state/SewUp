//! Bucket is an abstract concept to help you storage key value items.
//! The types of key and value should be specific when new a bucket.
//! Items save into bucket may have different encoding, the will base on the feature you enabled.
use std::cmp::PartialEq;
use std::marker::PhantomData;

use anyhow::Result;

use super::traits::{Key, Value, VecLike};
use crate::kv::traits::key::AsHashKey;
use crate::types::{Raw, Row};

/// `RawBucket` is a structure the data format really store
/// The hash key is stored in the first item, and the `Key` and `Value` are
/// stored in the second item
pub type RawBucket = (Vec<Raw>, Vec<Raw>);

/// Bucket is a wrapper for `RawBucket`, including the name of the bucket
pub struct Bucket<K: Key, V: Value> {
    pub(crate) name: String,
    pub(crate) raw_bucket: RawBucket,
    pub(crate) phantom_k: PhantomData<K>,
    pub(crate) phantom_v: PhantomData<V>,
}

type Item<K, V> = (K, V);

pub struct Iter<'a, K, V> {
    raw_bucket: &'a RawBucket,
    index: usize,
    item_idx: u32,
    upperlimit: Option<usize>,
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

impl<'a, K: Key, V: Value> Iterator for Iter<'a, K, V> {
    type Item = Item<K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let upperlimit = if let Some(upperlimit) = self.upperlimit {
                upperlimit
            } else {
                self.raw_bucket.0.len()
            };

            if self.index == upperlimit {
                return None;
            }

            let (k_size, v_size) = self.raw_bucket.0[self.index].get_size();
            self.index += 1;

            let mut key_row = Row::from(
                &self.raw_bucket.1[(self.item_idx) as usize..(self.item_idx + k_size) as usize],
            );

            key_row.make_buffer();
            let key = K::from_row_key(&key_row).expect("parse key from raw fail");

            let mut value_row = Row::from(
                &self.raw_bucket.1
                    [(self.item_idx + k_size) as usize..(self.item_idx + k_size + v_size) as usize],
            );
            value_row.make_buffer();
            let value = V::from_row_value(&value_row).expect("parse value from raw fail");

            self.item_idx = self.item_idx + k_size + v_size;

            return Some((key, value));
        }
    }
}

impl<'a, K: Key + PartialEq, V: Clone + Value> Bucket<K, V> {
    /// New a `Bucket` with name
    pub fn new(name: String, raw_bucket: RawBucket) -> Bucket<K, V> {
        Bucket {
            name,
            raw_bucket,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        }
    }

    fn bloom_filter(&self, hash: &[u8; 24]) -> bool {
        // TODO: bloom filter here
        true
    }

    /// Check the `Key` in the bucket
    pub fn contains(&self, key: K) -> Result<bool> {
        let hash = key.gen_hash()?;

        if !self.bloom_filter(&hash) {
            return Ok(false);
        }

        for item in self.raw_bucket.0.iter() {
            if item.get_size_from_hash(&hash).0 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get a `Value` from bucket by `Key`
    pub fn get(&self, key: K) -> Result<Option<V>> {
        let hash = key.gen_hash()?;

        let mut idx = 0u32;

        for item in self.raw_bucket.0.iter() {
            let (is_match, k_size, v_size) = item.get_size_from_hash(&hash);
            if is_match {
                let mut row = Row::from(
                    &self.raw_bucket.1[(idx + k_size) as usize..(idx + k_size + v_size) as usize],
                );
                row.make_buffer();
                let instance = V::from_row_value(&row)?;
                return Ok(Some(instance));
            }
            idx = idx + k_size + v_size;
        }
        Ok(None)
    }

    /// Set an item into the bucket
    pub fn set(&mut self, key: K, value: V) -> Result<()> {
        let mut value: Vec<Raw> = value.to_row_value()?.into();
        let mut row_key: Vec<Raw> = key.to_row_key()?.into();

        let hash_key = key.gen_hash_key(row_key.len() as u32, value.len() as u32)?;
        let hash = key.gen_hash()?;

        if self.bloom_filter(&hash) {
            let mut matched: Option<usize> = None;
            let mut idx = 0u32;
            for (i, item) in self.raw_bucket.0.iter().enumerate() {
                let (is_match, k_size, v_size) = item.get_size_from_hash(&hash);
                if is_match {
                    self.raw_bucket
                        .1
                        .drain((idx + k_size - 1) as usize..(idx + k_size + v_size) as usize);
                    matched = Some(i);
                    break;
                }
                idx = idx + k_size + v_size;
            }
            if let Some(i) = matched {
                self.raw_bucket.0.remove(i);
            }
        }

        self.raw_bucket.0.push(hash_key);
        self.raw_bucket.1.append(&mut row_key);
        self.raw_bucket.1.append(&mut value);

        Ok(())
    }

    /// Remove a item from the bucket by key
    pub fn remove(&mut self, key: K) -> Result<()> {
        let hash = key.gen_hash()?;

        let mut idx = 0u32;

        let mut matched: Option<usize> = None;
        for (i, item) in self.raw_bucket.0.iter().enumerate() {
            let (is_match, k_size, v_size) = item.get_size_from_hash(&hash);
            if is_match {
                self.raw_bucket
                    .1
                    .drain((idx + k_size - 1) as usize..(idx + k_size + v_size) as usize);
                matched = Some(i);
                break;
            }
            idx = idx + k_size + v_size;
        }
        if let Some(i) = matched {
            self.raw_bucket.0.remove(i);
        }
        Ok(())
    }

    /// Iterate all the items in the bucket
    pub fn iter(&self) -> Iter<K, V> {
        return Iter {
            raw_bucket: &self.raw_bucket,
            index: 0,
            item_idx: 0,
            upperlimit: None,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        };
    }

    /// Iterate the items in the bucket with specific range
    pub fn iter_range(&self, start: usize, end: usize) -> Iter<K, V> {
        return Iter {
            raw_bucket: &self.raw_bucket,
            index: start,
            item_idx: 0,
            upperlimit: Some(end),
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        };
    }

    /// Iterate the times with special prefix
    pub fn iter_prefix(&self, prefix: K) -> Iter<K, V> {
        unimplemented!();
    }

    // Always watch a Item among the blocks
    // pub fn watch(&self, key: K) -> Result<()> {
    //     unimplemented!();
    // }

    /// Get previous key, value pair
    pub fn prev_key(&self, key: K) -> Result<Option<Item<K, V>>> {
        unimplemented!();
    }

    /// Get next key value pair
    pub fn next_key(&self, niddle: K) -> Option<Item<K, V>> {
        let mut match_key = false;
        let mut index = 0;
        let mut item_idx = 0;
        loop {
            if index == self.raw_bucket.0.len() {
                return None;
            }

            let (k_size, v_size) = self.raw_bucket.0[index].get_size();

            let mut key_row =
                Row::from(&self.raw_bucket.1[(item_idx) as usize..(item_idx + k_size) as usize]);

            key_row.make_buffer();
            let key = K::from_row_key(&key_row).expect("parse key from raw fail");

            if match_key {
                let mut value_row = Row::from(
                    &self.raw_bucket.1
                        [(item_idx + k_size) as usize..(item_idx + k_size + v_size) as usize],
                );
                value_row.make_buffer();
                let value = V::from_row_value(&value_row).expect("parse value from raw fail");
                return Some((key, value));
            }

            if key == niddle {
                match_key = true;
            }

            index += 1;
            item_idx = item_idx + k_size + v_size;
        }
    }

    /// Pop item with specific key
    pub fn pop(&mut self, search_key: K) -> Option<V> {
        let mut index = 0;
        let mut item_idx = 0;
        loop {
            if index == self.raw_bucket.0.len() {
                return None;
            }

            let (k_size, v_size) = self.raw_bucket.0[index].get_size();

            let mut key_row =
                Row::from(&self.raw_bucket.1[(item_idx) as usize..(item_idx + k_size) as usize]);

            key_row.make_buffer();
            let key = K::from_row_key(&key_row).expect("parse key from raw fail");

            if key == search_key {
                let mut value_row = Row::from(
                    &self.raw_bucket.1
                        [(item_idx + k_size) as usize..(item_idx + k_size + v_size) as usize],
                );
                value_row.make_buffer();
                let value = V::from_row_value(&value_row).expect("parse value from raw fail");
                self.remove(key);
                return Some(value);
            }
            index += 1;
            item_idx = item_idx + k_size + v_size;
        }
    }

    /// Pop the last item
    pub fn pop_back(&mut self) -> Option<Item<K, V>> {
        let mut prev_pair: Option<(K, V)> = None;
        let mut iter = self.iter();
        while let Some(pair) = iter.next() {
            prev_pair = Some(pair);
        }
        if let Some((key, value)) = prev_pair {
            self.remove(key.clone());
            Some((key, value))
        } else {
            None
        }
    }

    /// Pop the first item
    pub fn pop_front(&mut self) -> Option<Item<K, V>> {
        if let Some((key, value)) = self.iter().next() {
            self.remove(key.clone());
            Some((key, value))
        } else {
            None
        }
    }

    /// Get the length of the bucket
    pub fn len(&self) -> usize {
        self.raw_bucket.0.len()
    }

    /// Check there is something in the bucket
    pub fn is_empty(&self) -> bool {
        self.raw_bucket.0.is_empty()
    }
}

pub type SewUpVec<T> = super::bucket::Bucket<usize, T>;

impl<V: Clone + Value> SewUpVec<V> {
    fn from_vec(&mut self, v: Vec<V>) {
        *self = super::bucket::Bucket::<usize, V>::new(self.name.clone(), (Vec::new(), Vec::new()));
        v.into_iter().enumerate().map(|(i, v)| self.set(i, v));
    }
}

// TODO: check thiese function can be done without from_vec method
impl<V: Clone + Value + PartialEq> VecLike<V> for SewUpVec<V> {
    fn to_vec(&self) -> Vec<V> {
        self.iter().map(|(_, v)| v).collect()
    }

    fn append(&mut self, other: &mut Vec<V>) {
        for (i, v) in other.into_iter().enumerate() {
            self.set(self.len() + i, v.clone());
        }
    }

    fn push(&mut self, value: V) {
        self.set(self.len(), value);
    }

    fn pop(&mut self) -> Option<V> {
        // TODO refactor this when `pop` of bucket implemented
        let len = self.len();
        if len == 0 {
            None
        } else {
            let v = self.get(len - 1).expect("there should be value");
            self.remove(len - 1);
            v
        }
    }

    fn clear(&mut self) {
        *self = super::bucket::Bucket::<usize, V>::new(self.name.clone(), (Vec::new(), Vec::new()));
    }

    fn resize_with<F>(&mut self, new_len: usize, mut f: F)
    where
        F: FnMut() -> V,
    {
        let len = self.len();
        if new_len > len {
            for i in len..new_len {
                self.set(i, f());
            }
        } else {
            self.truncate(new_len);
        }
    }

    fn resize(&mut self, new_len: usize, value: V) {
        let len = self.len();
        if new_len > len {
            for i in len..new_len {
                self.set(i, value.clone());
            }
        } else {
            self.truncate(new_len);
        }
    }

    fn extend_from_slice(&mut self, other: &[V]) {
        for (i, v) in other.into_iter().enumerate() {
            self.set(self.len() + i, v.clone());
        }
    }

    fn dedup(&mut self) {
        let mut v = self.to_vec();
        v.dedup();
        self.from_vec(v);
    }

    fn swap(&mut self, a: usize, b: usize) {
        let tmp = self.get(a).expect("swap index should exist in Vec");
        assert!(b < self.len());
        self.set(b, tmp.expect("swap instance should exist"));
    }

    fn reverse(&mut self) {
        let mut v = self.to_vec();
        v.reverse();
        self.from_vec(v);
    }

    // TODO: add bloom filter field for SewUpVec
    fn contains(&self, x: &V) -> bool {
        for (_, v) in self.iter() {
            if v == *x {
                return true;
            }
        }
        false
    }

    fn starts_with(&self, needle: &[V]) -> bool {
        self.to_vec().starts_with(needle)
    }

    fn ends_with(&self, needle: &[V]) -> bool {
        self.to_vec().ends_with(needle)
    }

    fn rotate_left(&mut self, mid: usize) {
        let mut v = self.to_vec();
        v.rotate_left(mid);
        self.from_vec(v);
    }

    fn rotate_right(&mut self, k: usize) {
        let mut v = self.to_vec();
        v.rotate_right(k);
        self.from_vec(v);
    }

    fn fill_with<F>(&mut self, mut f: F)
    where
        F: FnMut() -> V,
    {
        for i in 0..self.len() {
            self.set(i, f());
        }
    }

    fn copy_from_slice(&mut self, src: &[V])
    where
        V: Copy,
    {
        let mut v = self.to_vec();
        v.copy_from_slice(src);
        self.from_vec(v);
    }

    fn sort(&mut self)
    where
        V: Ord,
    {
        let mut v = self.to_vec();
        v.sort();
        self.from_vec(v);
    }

    fn truncate(&mut self, len: usize) {
        if len > self.len() {
            return;
        }
        for i in self.len()..len {
            self.remove(i);
        }
    }
}
