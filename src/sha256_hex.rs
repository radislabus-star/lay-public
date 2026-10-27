/// Lowercase, fixed-width SHA-256 representation independent of digest's array API.
pub(crate) fn encode(value: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(64);
    for byte in value {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::encode;
    use sha2::{Digest, Sha256};

    #[test]
    fn sha256_hex_preserves_all_bytes_and_leading_zeroes() {
        for byte in 0..=u8::MAX {
            assert_eq!(encode([byte; 32]), format!("{byte:02x}").repeat(32));
        }
    }

    #[test]
    fn sha256_vectors_remain_stable() {
        assert_eq!(
            encode(Sha256::digest(b"").into()),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            encode(Sha256::digest(b"abc").into()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
