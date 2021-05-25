use anyhow::Result;
use std::any::Any as StdAny;

/// This is temp struct will be changed after implement
type Any = Box<dyn StdAny>;
/// This is temp struct will be changed after implement
type Item<K, V> = (K, V);
/// This is temp struct will be changed after implement
type Iter<K, V> = Vec<(K, V)>;

pub struct Bucket {}

impl Bucket {
    pub fn new() -> Bucket {
        Bucket {}
    }

    pub fn contains(&self, _key: Any) -> Result<bool> {
        Ok(true)
    }

    pub fn get(&self, _key: Any) -> Result<Option<Any>> {
        Ok(None)
    }

    pub fn set(&self, _key: Any, _value: Any) -> Result<()> {
        Ok(())
    }

    pub fn remove(&self, _key: Any) -> Result<()> {
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
        0
    }

    pub fn is_empty(&self) -> bool {
        true
    }
}
