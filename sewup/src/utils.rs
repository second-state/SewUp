use tiny_keccak::{Hasher, Keccak, Sha3};

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

pub fn get_function_signature(function_prototype: &str) -> [u8; 4] {
    let mut sig = [0; 4];
    let mut hasher = Keccak::v256();
    hasher.update(function_prototype.as_bytes());
    hasher.finalize(&mut sig);
    sig
}

#[inline]
pub fn storage_index_to_addr(idx: usize, addr: &mut [u8; 32]) {
    for j in 0..(idx / 32) + 1 {
        assert!(j < 32, "Too big to store on chain");
        addr[j] = (idx >> (5 * j) & 31) as u8;
    }
}

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
