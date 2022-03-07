use std::iter::Iterator;
use std::marker::PhantomData;

use anyhow::Result;
use ewasm_api::storage_store;
use serde::Serialize;
use serde_derive::Serialize as SerializeDerive;

use crate::utils::storage_index_to_addr;

const CONFIG_ADDR: [u8; 32] = [0; 32];

pub trait SingleBucket
where
    Self: Serialize + Default,
{
    fn commit(&self) -> Result<u32> {
        let mut buffer = [0u8; 32];
        let bin = bincode::serialize(&self).expect("serialize db binary fail");
        let length = bin.len();

        let mut len_buffer = bin.len().to_be_bytes();
        len_buffer.swap_with_slice(&mut buffer[28..32]);

        storage_store(&CONFIG_ADDR.into(), &buffer.into());

        let mut addr: [u8; 32] = [0; 32];
        let mut storage_index = 0;
        let mut iter = bin.chunks_exact(32);
        while storage_index * 32 < length as usize {
            storage_index += 1;
            storage_index_to_addr(storage_index, &mut addr);

            if let Some(chunk) = iter.next() {
                let part: [u8; 32] = chunk.try_into().unwrap();
                storage_store(&addr.into(), &part.into());
            } else {
                let remainder = iter.remainder();
                storage_index_to_addr(storage_index, &mut addr);
                let mut part = [0u8; 32];
                for i in 0..length & 31 {
                    part[i] = remainder[i];
                }
                storage_store(&addr.into(), &part.into());
                break;
            }
        }
        Ok(length as u32)
    }
}

pub struct Item<K, V> {
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

pub enum Pair1<K, V> {
    Item1(Item<K, V>),
}

pub trait SingleBucket1<K1, V1>: SingleBucket + Default {
    type Pairs: Iterator<Item = Pair1<K1, V1>>;
}

pub enum Pair2<K1, V1, K2, V2> {
    Item1(Item<K1, V1>),
    Item2(Item<K2, V2>),
}

#[derive(Default, SerializeDerive)]
pub struct SingleBucket2<K1: Default, V1: Default, K2: Default, V2: Default> {
    phantom_k1: PhantomData<K1>,
    phantom_v1: PhantomData<V1>,
    phantom_k2: PhantomData<K2>,
    phantom_v2: PhantomData<V2>,
}
impl<K1: Default, V1: Default, K2: Default, V2: Default> SingleBucket
    for SingleBucket2<K1, V1, K2, V2>
{
}
