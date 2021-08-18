#[cfg(target_arch = "wasm32")]
use ewasm_api::finish_data;
#[cfg(target_arch = "wasm32")]
use ewasm_api::log0;
use tiny_keccak::{Hasher, Sha3};

pub use serde::de::DeserializeOwned;
pub use serde::Serialize;
pub use serde_value::{to_value, Value};

#[cfg(target_arch = "wasm32")]
use crate::types::Raw;

/// helps you debug the ewasm contract when executing in the test runtime
/// To show the debug message pllease run the test case as following command
/// `cargo test -- --nocapture`
/// Or you may checkout the log file set by following `ewasm_test` macro
/// `#[ewasm_test(log=/tmp/default.log)]`
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! ewasm_dbg {
    () => { };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::utils::log(format!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp));
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::ewasm_dbg!($val)),+,)
    };
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! ewasm_dbg {
    () => {};
}

#[cfg(target_arch = "wasm32")]
pub fn log(s: String) {
    log0(s.as_bytes());
}

#[cfg(target_arch = "wasm32")]
pub fn ewasm_return(bytes: Vec<u8>) {
    finish_data(&bytes);
}

#[cfg(target_arch = "wasm32")]
pub fn ewasm_return_str(s: &str) {
    let mut output = Raw::from(32u32).as_bytes().to_vec();
    output.append(&mut Raw::from(s.len()).as_bytes().to_vec());
    output.append(&mut Raw::from(s).as_bytes().to_vec());
    finish_data(&output);
}

#[cfg(target_arch = "wasm32")]
pub fn ewasm_return_vec(v: &Vec<[u8; 32]>) {
    let mut output = Raw::from(32u32).as_bytes().to_vec();
    output.append(&mut Raw::from(v.len()).as_bytes().to_vec());
    for e in v.iter() {
        output.extend_from_slice(e);
    }
    finish_data(&output);
}

#[cfg(target_arch = "wasm32")]
pub fn ewasm_return_bool(is_true: bool) {
    let output = if is_true {
        vec![
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8,
        ]
    } else {
        vec![
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ]
    };
    finish_data(&output);
}

pub fn copy_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Copy,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
    a
}

pub fn sha3_256(input: &[u8]) -> [u8; 32] {
    let mut output = [0; 32];
    let mut hasher = Sha3::v256();
    hasher.update(input);
    hasher.finalize(&mut output);
    output
}

#[inline]
pub fn storage_index_to_addr(idx: usize, addr: &mut [u8; 32]) {
    for (j, byte) in addr.iter_mut().enumerate().take((idx / 32) + 1) {
        assert!(j < 32, "Too big to store on chain");
        *byte = (idx >> (5 * j) & 31) as u8;
    }
}

pub fn get_field_by_name<T, R>(data: T, field: &str) -> R
where
    T: Serialize,
    R: DeserializeOwned,
{
    let mut map = match to_value(data) {
        Ok(Value::Map(map)) => map,
        _ => panic!("expected a struct"),
    };

    let key = Value::String(field.to_owned());
    let value = match map.remove(&key) {
        Some(value) => value,
        None => panic!("no such field"),
    };

    match R::deserialize(value) {
        Ok(r) => r,
        Err(_) => panic!("type incorrect"),
    }
}

#[cfg(feature = "default")]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_storage_index_to_addr() {
        let mut addr: [u8; 32] = [0; 32];

        storage_index_to_addr(1, &mut addr);
        assert_eq!(
            addr,
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        storage_index_to_addr(32, &mut addr);
        assert_eq!(
            addr,
            [
                0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        storage_index_to_addr(33, &mut addr);
        assert_eq!(
            addr,
            [
                1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        storage_index_to_addr(65, &mut addr);
        assert_eq!(
            addr,
            [
                1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );
    }
}
