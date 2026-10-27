pub(crate) fn mix64_golden(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    mix64_avalanche(value)
}

pub(crate) fn mix64_avalanche(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

/// Lowercase, fixed-width SHA-256 representation independent of digest's array API.
pub(crate) fn encode_sha256_hex(value: [u8; 32]) -> String {
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
    use super::encode_sha256_hex;
    use sha2::{Digest, Sha256};

    #[test]
    fn sha256_hex_preserves_all_bytes_and_leading_zeroes() {
        for byte in 0..=u8::MAX {
            assert_eq!(encode_sha256_hex([byte; 32]), format!("{byte:02x}").repeat(32));
        }
    }

    #[test]
    fn sha256_vectors_remain_stable() {
        assert_eq!(
            encode_sha256_hex(Sha256::digest(b"").into()),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            encode_sha256_hex(Sha256::digest(b"abc").into()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
