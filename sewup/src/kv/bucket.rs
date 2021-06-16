use std::marker::PhantomData;

use anyhow::Result;

use super::traits::{Key, Value};
use crate::kv::traits::key::AsHashKey;
use crate::types::{Raw, Row};

// TODO: quick for first iteration
pub type RawBucket = (Vec<Raw>, Vec<Raw>);

/// Bucket is an abstract concept to help you storage key value items.
/// The types of key and value should be specific when new a bucket.
/// Items save into bucket may have different encoding, the will base on the
/// feature you enabled.
pub struct Bucket<K: Key, V: Value> {
    pub(crate) name: String,
    pub(crate) raw_bucket: RawBucket,
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
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

            let mut key_row = Row::from(
                &self.raw_bucket.1[(self.item_idx) as usize..(self.item_idx + k_size) as usize],
            );

            key_row.make_buffer();
            let key = K::from_raw_key(&key_row).expect("parse key from raw fail");

            let mut value_row = Row::from(
                &self.raw_bucket.1
                    [(self.item_idx + k_size) as usize..(self.item_idx + k_size + v_size) as usize],
            );
            value_row.make_buffer();
            let value = V::from_raw_value(&value_row).expect("parse value from raw fail");

            self.item_idx = self.item_idx + k_size + v_size;

            return Some((key, value));
        }
    }
}

impl<'a, K: Key, V: Clone + Value> Bucket<K, V> {
    pub fn new(name: String, raw_bucket: RawBucket) -> Bucket<K, V> {
        Bucket {
            name,
            raw_bucket,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        }
    }

    pub fn contains(&self, key: K) -> Result<bool> {
        let hash = key.gen_hash()?;

        // TODO: bloom filter here

        for item in self.raw_bucket.0.iter() {
            if item.get_size_from_hash(hash).0 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn get(&self, key: K) -> Result<Option<V>> {
        let hash = key.gen_hash()?;

        let mut idx = 0u32;

        for item in self.raw_bucket.0.iter() {
            let (is_match, k_size, v_size) = item.get_size_from_hash(hash);
            if is_match {
                let mut row = Row::from(
                    &self.raw_bucket.1[(idx + k_size) as usize..(idx + k_size + v_size) as usize],
                );
                row.make_buffer();
                let instance = V::from_raw_value(&row)?;
                return Ok(Some(instance));
            }
            idx = idx + k_size + v_size;
        }
        Ok(None)
    }

    pub fn set(&mut self, key: K, value: V) -> Result<()> {
        let mut value: Vec<Raw> = value.to_raw_value()?.into();
        let mut raw_key: Vec<Raw> = key.to_raw_key()?.into();

        let hash_key = key.gen_hash_key(raw_key.len() as u32, value.len() as u32)?;

        // TODO: handle key duplicate here

        self.raw_bucket.0.push(hash_key);
        self.raw_bucket.1.append(&mut raw_key);
        self.raw_bucket.1.append(&mut value);

        Ok(())
    }

    pub fn remove(&mut self, key: K) -> Result<()> {
        let hash = key.gen_hash()?;

        let mut idx = 0u32;

        for (i, item) in self.raw_bucket.0.iter().enumerate() {
            let (is_match, k_size, v_size) = item.get_size_from_hash(hash);
            if is_match {
                // TODO: better implement here
                for _ in (idx + k_size)..(idx + k_size + v_size) {
                    self.raw_bucket.1.remove(idx as usize);
                }
                idx = i as u32;
                break;
            }
            idx = idx + k_size + v_size;
        }
        self.raw_bucket.0.remove(idx as usize);
        Ok(())
    }

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

    /// May not work
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

    /// May not work
    pub fn iter_prefix(&self, prefix: K) -> Iter<K, V> {
        unimplemented!();
    }

    /// Native only, return an watch object, May not work
    pub fn watch(&self, key: K) -> Result<()> {
        unimplemented!();
    }

    /// Get previous key, value pair
    pub fn prev_key(&self, key: K) -> Result<Option<Item<K, V>>> {
        unimplemented!();
    }

    /// Get next key value paire
    pub fn next_key(&self, key: K) -> Result<Option<Item<K, V>>> {
        unimplemented!();
    }

    /// Pop items
    pub fn pop(&self, key: K) -> Result<Option<V>> {
        unimplemented!();
    }

    /// Pop the last item
    pub fn pop_back(&self) -> Result<Option<Item<K, V>>> {
        Ok(None)
    }

    /// Pop the first item
    pub fn pop_front(&self) -> Result<Option<Item<K, V>>> {
        Ok(None)
    }

    pub fn len(&self) -> usize {
        self.raw_bucket.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw_bucket.0.is_empty()
    }
}
