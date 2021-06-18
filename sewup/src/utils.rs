use tiny_keccak::{Hasher, Sha3};

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
