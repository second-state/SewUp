use crate::types::*;

#[cfg(feature = "default")]
#[test]
fn test_serde_for_raw() {
    let raw = Raw::from(vec![0, 1]);
    assert_eq!(
        raw.bytes,
        [
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0
        ]
    );
    let bin = bincode::serialize(&raw).expect("serialize raw fail");
    let load: Raw = bincode::deserialize(&bin).expect("load raw binary fail");
    assert_eq!(raw.bytes, load.bytes);
    // assert_eq!(0, raw.flag);
    // assert_eq!(1, load.flag);
}

#[cfg(feature = "default")]
#[test]
fn test_serde_for_raw2() {
    let raw = Raw::from(vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 200, 201, 202, 203, 204, 205, 206,
        207, 208, 209, 210, 211, 212, 213, 214, 215,
    ]);
    assert_eq!(
        raw.bytes,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 200, 201, 202, 203, 204, 205,
            206, 207, 208, 209, 210, 211, 212, 213, 214, 215
        ]
    );
    let bin = bincode::serialize(&raw).expect("serialize raw fail");
    let load: Raw = bincode::deserialize(&bin).expect("load raw binary fail");
    assert_eq!(raw.bytes, load.bytes);
    // assert_eq!(0, raw.flag);
    // assert_eq!(1, load.flag);
}

#[cfg(feature = "default")]
#[test]
fn test_from() {
    let r1 = Raw::from(vec![1, 2, 3]);
    assert_eq!(
        r1,
        vec![
            1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0
        ]
    );
    let r2 = Raw::from(&[4; 32][..]);
    assert_eq!(r2, vec![4; 32]);
}

#[cfg(feature = "default")]
#[test]
fn test_short_string() {
    // TODO: need more design on string
    let r1 = Raw::from("abcd");
    assert_eq!(
        r1,
        vec![
            97, 98, 99, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0
        ]
    );
}

#[cfg(feature = "default")]
#[test]
fn test_box() {
    let box1: Box<[u8]> = Box::new([1, 2, 3]);
    let r1: Raw = box1.into();
    assert_eq!(
        r1,
        vec![
            1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0
        ]
    );
    let box2: Box<[u8]> = Box::new([5; 32]);
    let r2: Raw = box2.into();
    assert_eq!(r2, vec![5; 32]);
}

#[cfg(feature = "default")]
#[test]
fn test_serde_for_row() {
    let row = Row::from(vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 200, 201, 202, 203, 204, 205, 206,
        207, 208, 209, 210, 211, 212, 213, 214, 215, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
        14, 15,
    ]);
    assert_eq!(
        row.inner,
        vec![
            Raw::from(vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 200, 201, 202, 203, 204, 205,
                206, 207, 208, 209, 210, 211, 212, 213, 214, 215
            ]),
            Raw::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,])
        ]
    );
    let bin = bincode::serialize(&row).expect("serialize raw fail");
    let load: Row = bincode::deserialize(&bin).expect("load raw binary fail");
    assert_eq!(row.inner, load.inner);
}

#[cfg(feature = "default")]
#[test]
fn test_str_for_row() {
    let row =
        Row::from("Life is like riding a bicycle. To keep your balance, you must keep moving.");
    assert_eq!(
        row.inner,
        vec![
            [
                76, 105, 102, 101, 32, 105, 115, 32, 108, 105, 107, 101, 32, 114, 105, 100, 105,
                110, 103, 32, 97, 32, 98, 105, 99, 121, 99, 108, 101, 46, 32, 84
            ],
            [
                111, 32, 107, 101, 101, 112, 32, 121, 111, 117, 114, 32, 98, 97, 108, 97, 110, 99,
                101, 44, 32, 121, 111, 117, 32, 109, 117, 115, 116, 32, 107, 101
            ],
            [
                101, 112, 32, 109, 111, 118, 105, 110, 103, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        ]
    );
}

#[cfg(feature = "default")]
#[test]
fn test_integer_raw_convert() {
    let i = 5u8;
    let raw = Raw::from(i);
    let j = u8::from(raw);
    assert_eq!(i, j);

    let i = 300u16;
    let raw = Raw::from(i);
    let j = u16::from(raw);
    assert_eq!(i, j);

    let i = 4294967295u32;
    let raw = Raw::from(i);
    let j = u32::from(raw);
    assert_eq!(i, j);

    let i = 4300000000u64;
    let raw = Raw::from(i);
    let j = u64::from(raw);
    assert_eq!(i, j);

    let i = 4294967295usize;
    let raw = Raw::from(i);
    let j = usize::from(raw);
    assert_eq!(i, j);

    let i = -5i8;
    let raw = Raw::from(i);
    let j = i8::from(raw);
    assert_eq!(i, j);

    let i = -300i16;
    let raw = Raw::from(i);
    let j = i16::from(raw);
    assert_eq!(i, j);

    let i = -2147483648i32;
    let raw = Raw::from(i);
    let j = i32::from(raw);
    assert_eq!(i, j);

    let i = -4294967295i64;
    let raw = Raw::from(i);
    let j = i64::from(raw);
    assert_eq!(i, j);

    let i = -2147483648isize;
    let raw = Raw::from(i);
    let j = isize::from(raw);
    assert_eq!(i, j);
}
