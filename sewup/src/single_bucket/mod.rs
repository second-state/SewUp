use std::iter::Iterator;
use std::marker::PhantomData;

pub struct Item<K, V> {
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

pub enum Pair<K, V> {
    Item1(Item<K, V>),
}

pub trait SingleBucket<K1, V1> {
    type Pairs: Iterator<Item = Pair<K1, V1>>;
}
