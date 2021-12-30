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
    let front = bucket.pop_front().unwrap();
    assert_eq!(front, Some((1, 1)));
    assert_eq!(bucket.get(1).unwrap(), None);
}
