//! Bucket is an abstract concept to help you storage key value items.
//! The types of key and value should be specific when new a bucket.
//! Items save into bucket may have different encoding, the will base on the feature you enabled.
use std::marker::PhantomData;

use anyhow::Result;

use super::traits::{Key, Value};
use crate::kv::traits::key::AsHashKey;
use crate::types::{Raw, Row};

// TODO: quick for first iteration
/// `RawBucket` is a structure the data format really store
/// The hash key is stored in the first item, and the `Key` and `Value` are
/// stored in the second item
pub type RawBucket = (Vec<Raw>, Vec<Raw>);

/// Bucket is a wrapper for `RawBucket`, including the name of the bucket
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

impl<'a, K: Key, V: Clone + Value> Bucket<K, V> {
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
    pub fn next_key(&self, key: K) -> Result<Option<Item<K, V>>> {
        unimplemented!();
    }

    /// Pop item with specific key
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

impl<'a, V: Clone + Value> SewUpVec<V> {
    //pub fn to_vec(&self) -> Vec<V>
    //
    //pub fn append(&mut self, other: &mut Vec<T, A>)
    //pub fn drain<R>(&mut self, range: R) -> Drain<'_, T, A>
    //pub fn clear(&mut self)
    //pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    //pub fn resize(&mut self, new_len: usize, value: T)
    //pub fn extend_from_slice(&mut self, other: &[T])
    //pub fn dedup(&mut self)
    //pub fn first(&self) -> Option<&T>
    //pub fn first_mut(&mut self) -> Option<&mut T>
    //
    //pub fn last(&self) -> Option<&T>
    //pub fn last_mut(&mut self) -> Option<&mut T>
    //pub fn get_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[T]>>::Output>
    //
    //pub fn swap(&mut self, a: usize, b: usize)
    //pub fn reverse(&mut self)
    //pub fn windows(&self, size: usize) -> Windows<'_, T>
    //pub fn chunks(&self, chunk_size: usize) -> Chunks<'_, T>
    //pub fn chunks_mut(&mut self, chunk_size: usize) -> ChunksMut<'_, T>
    //pub fn chunks_exact(&self, chunk_size: usize) -> ChunksExact<'_, T>
    //pub fn chunks_exact_mut(&mut self, chunk_size: usize) -> ChunksExactMut<'_, T>
    //pub fn contains(&self, x: &T) -> bool
    //pub fn starts_with(&self, needle: &[T]) -> bool
    //pub fn ends_with(&self, needle: &[T]) -> bool
    //pub fn strip_prefix<P>(&self, prefix: &P) -> Option<&[T]>
    //pub fn strip_suffix<P>(&self, suffix: &P) -> Option<&[T]>
    //pub fn rotate_left(&mut self, mid: usize)
    //pub fn rotate_right(&mut self, k: usize)
    //pub fn fill_with<F>(&mut self, f: F)
    //pub fn copy_from_slice(&mut self, src: &[T]) where T: Copy,
    //pub fn sort(&mut self) where T: Ord,
    //pub fn concat<Item>(&self) -> <[T] as Concat<Item>>::Output
    //pub fn join<Separator>(&self, sep: Separator) -> <[T] as Join<Separator>>::Output
}
