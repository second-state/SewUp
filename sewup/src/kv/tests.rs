use std::marker::PhantomData;

use crate::kv::*;

#[cfg(feature = "default")]
#[test]
fn test_pop_front_for_bucket() {
    let mut bucket = Bucket {
        name: "test_bucket".into(),
        raw_bucket: (vec![], vec![]),
        phantom_k: PhantomData::<usize>,
        phantom_v: PhantomData::<usize>,
    };
    bucket.set(1, 1);
    bucket.set(2, 2);
    assert_eq!(bucket.get(1).unwrap(), Some(1));
    let front = bucket.pop_front();
    assert_eq!(front, Some((1, 1)));
    assert_eq!(bucket.get(1).unwrap(), None);
    assert_eq!(bucket.get(2).unwrap(), Some(2));
}

#[cfg(feature = "default")]
#[test]
fn test_pop_for_bucket() {
    let mut bucket = Bucket {
        name: "test_bucket".into(),
        raw_bucket: (vec![], vec![]),
        phantom_k: PhantomData::<usize>,
        phantom_v: PhantomData::<usize>,
    };
    bucket.set(1, 1);
    bucket.set(2, 2);
    bucket.set(3, 3);
    assert_eq!(bucket.get(1).unwrap(), Some(1));
    let value = bucket.pop(2);
    assert_eq!(value, Some(2));
    assert_eq!(bucket.get(2).unwrap(), None);
    assert_eq!(bucket.get(3).unwrap(), Some(3));
}

#[cfg(feature = "default")]
#[test]
fn test_pop_back_for_bucket() {
    let mut bucket = Bucket {
        name: "test_bucket".into(),
        raw_bucket: (vec![], vec![]),
        phantom_k: PhantomData::<usize>,
        phantom_v: PhantomData::<usize>,
    };
    bucket.set(1, 1);
    bucket.set(2, 2);
    assert_eq!(bucket.get(1).unwrap(), Some(1));
    let last = bucket.pop_back();
    assert_eq!(last, Some((2, 2)));
    assert_eq!(bucket.get(1).unwrap(), Some(1));
    assert_eq!(bucket.get(2).unwrap(), None);
}
