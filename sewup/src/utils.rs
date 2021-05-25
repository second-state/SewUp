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

pub fn get_function_signature(funtion_prototype: &str) -> [u8; 4] {
    let mut sig = [0; 4];
    let mut hasher = Keccak::v256();
    hasher.update(funtion_prototype.as_bytes());
    hasher.finalize(&mut sig);
    sig
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::*;
    #[test]
    fn test_function_signature() {
        let mut sig: [u8; 4] = hex!("c48d6d5e");
        assert_eq!(get_function_signature("sendMessage(string,address)"), sig);
        sig = hex!("70a08231");
        assert_eq!(get_function_signature("balanceOf(address)"), sig);
    }
}
