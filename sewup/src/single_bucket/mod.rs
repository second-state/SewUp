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

pub enum Pair2<K1, V1, K2, V2> {
    Item1(Item<K1, V1>),
    Item2(Item<K2, V2>),
}

macro_rules! single_bucket_factory {
    ($i:expr, ($($o:ident),+)) => {
        paste::paste! {
            #[derive(Default, SerializeDerive)]
            #[allow(non_snake_case)]
            pub struct [< SingleBucket $i >]<$($o: Default),+> {
                $($o: PhantomData<$o>),+
            }
            impl<$($o: Default),+> SingleBucket for [<  SingleBucket $i >] <$($o),+> {

            }
        }
    }
}

single_bucket_factory!(1, (K1, V1));
single_bucket_factory!(2, (K1, V1, K2, V2));
single_bucket_factory!(3, (K1, V1, K2, V2, K3, V3));
single_bucket_factory!(4, (K1, V1, K2, V2, K3, V3, K4, V4));
single_bucket_factory!(5, (K1, V1, K2, V2, K3, V3, K4, V4, K5, V5));
single_bucket_factory!(6, (K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6));
single_bucket_factory!(7, (K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7));
single_bucket_factory!(
    8,
    (K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8)
);
single_bucket_factory!(
    9,
    (K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9)
);
single_bucket_factory!(
    10,
    (K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10)
);
single_bucket_factory!(
    11,
    (K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11)
);
single_bucket_factory!(
    12,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12
    )
);
single_bucket_factory!(
    13,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13
    )
);
single_bucket_factory!(
    14,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14
    )
);
single_bucket_factory!(
    15,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15
    )
);
single_bucket_factory!(
    16,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16
    )
);
single_bucket_factory!(
    17,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17
    )
);
single_bucket_factory!(
    18,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18
    )
);
single_bucket_factory!(
    19,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19
    )
);
single_bucket_factory!(
    20,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20
    )
);
single_bucket_factory!(
    21,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21
    )
);
single_bucket_factory!(
    22,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22
    )
);
single_bucket_factory!(
    23,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23
    )
);
single_bucket_factory!(
    24,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24
    )
);
single_bucket_factory!(
    25,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24, K25, V25
    )
);
single_bucket_factory!(
    26,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24, K25, V25, K26, V26
    )
);
single_bucket_factory!(
    27,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24, K25, V25, K26, V26, K27, V27
    )
);
single_bucket_factory!(
    28,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24, K25, V25, K26, V26, K27, V27, K28, V28
    )
);
single_bucket_factory!(
    29,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24, K25, V25, K26, V26, K27, V27, K28, V28, K29, V29
    )
);
single_bucket_factory!(
    30,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24, K25, V25, K26, V26, K27, V27, K28, V28, K29, V29,
        K30, V30
    )
);
single_bucket_factory!(
    31,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24, K25, V25, K26, V26, K27, V27, K28, V28, K29, V29,
        K30, V30, K31, V31
    )
);
single_bucket_factory!(
    32,
    (
        K1, V1, K2, V2, K3, V3, K4, V4, K5, V5, K6, V6, K7, V7, K8, V8, K9, V9, K10, V10, K11, V11,
        K12, V12, K13, V13, K14, V14, K15, V15, K16, V16, K17, V17, K18, V18, K19, V19, K20, V20,
        K21, V21, K22, V22, K23, V23, K24, V24, K25, V25, K26, V26, K27, V27, K28, V28, K29, V29,
        K30, V30, K31, V31, K32, V32
    )
);
