use std::any::Any as StdAny;
use std::marker::PhantomData;

use anyhow::Result;

use super::traits::{Key, Value};
use crate::kv::traits::key::AsHashKey;
use crate::types::{Raw, Row};

// TODO: quick for first iteration
pub type RawBucket = (Vec<Raw>, Vec<Raw>);

/// This is temp struct will be changed after implement
type Any = Box<dyn StdAny>;
/// This is temp struct will be changed after implement
type Item<K, V> = (Row, Row, PhantomData<K>, PhantomData<V>);
/// This is temp struct will be changed after implement
type Iter<K, V> = Vec<(K, V)>;

/// Bucket is an abstract concept to help you storage key value items.
/// The types of key and value should be specific when new a bucket.
/// Items save into bucket may have different encoding, the will base on the
/// feature you enabled.
pub struct Bucket<'a, K: Key<'a>, V: Value<'a>> {
    pub(crate) name: String,
    pub(crate) raw_bucket: RawBucket,
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
    phantom: PhantomData<&'a ()>,
}

impl<'a, K: Key<'a>, V: Clone + Value<'a>> Bucket<'a, K, V> {
    pub fn new(name: String, raw_bucket: RawBucket) -> Bucket<'a, K, V> {
        Bucket {
            name,
            raw_bucket,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
            phantom: PhantomData,
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

    pub fn get(&mut self, key: K) -> Result<Option<V>> {
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

    pub fn iter(&self) -> Iter<Any, Any> {
        unimplemented!();
    }

    /// May not work
    pub fn iter_range(&self, a: Any, b: Any) -> Iter<Any, Any> {
        unimplemented!();
    }

    /// May not work
    pub fn iter_prefix(&self, a: Any) -> Iter<Any, Any> {
        unimplemented!();
    }

    /// Native only, return an watch object, May not work
    pub fn watch(&self, key: Any) -> Result<()> {
        unimplemented!();
    }

    /// Get previous key, value pair
    pub fn prev_key(&self, key: Any) -> Result<Option<Item<Any, Any>>> {
        unimplemented!();
    }

    /// Get next key value paire
    pub fn next_key(&self, key: Any) -> Result<Option<Item<Any, Any>>> {
        unimplemented!();
    }

    /// Pop items
    pub fn pop(&self, key: Any) -> Result<Option<Any>> {
        unimplemented!();
    }

    /// Pop the last item
    pub fn pop_back(&self) -> Result<Option<Item<Any, Any>>> {
        Ok(None)
    }

    /// Pop the first item
    pub fn pop_front(&self) -> Result<Option<Item<Any, Any>>> {
        Ok(None)
    }

    pub fn len(&self) -> usize {
        self.raw_bucket.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw_bucket.0.is_empty()
    }
}
