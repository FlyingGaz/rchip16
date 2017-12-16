use std::mem;
use std::slice;

/// Read a single value from a bitflag
pub fn bitflag(flag: u8, index: u8) -> bool {
    flag & 1 << index != 0
}

/// Set a single value in a bitflag
pub fn set_bitflag(flag: &mut u8, index: u8, value: bool) {
    if value {
        *flag |= 1 << index
    } else {
        *flag &= !(1 << index)
    }
}

/// Split a byte into the `(high, low)` half-bytes
pub fn half_bytes(byte: u8) -> (u8, u8) {
    (byte >> 4, byte & 0x0F)
}

/// Serialize a value to a slice of bytes.
pub fn serialize<T>(src: &T) -> &[u8] {
    unsafe { slice::from_raw_parts(mem::transmute::<_, *const u8>(src), mem::size_of::<T>()) }
}

/// Deserialize a value from a slice of bytes.
/// This function will panic if the slice is not long enough.
pub fn deserialize<T>(src: &[u8]) -> &T {
    let len = src.len();
    let size = mem::size_of::<T>();
    assert!(len >= size, "src not big enough: the len is {} but the size of T is {}", len, size);
    unsafe { mem::transmute::<_, &T>(src.as_ptr()) }
}

/// Calculate the CRC32 checksum of a buffer
pub fn crc32(stream: &[u8]) -> u32 {
    let mask = 0xEDB88320;
    let mut checksum = !0;
    for &byte in stream {
        for i in 0..8 {
            if checksum & 1 != (byte as u32 >> i) & 1 {
                checksum = (checksum >> 1) ^ mask;
            } else {
                checksum = checksum >> 1;
            }
        }
    }
    !checksum
}

#[cfg(test)]
mod tests {
    use util::*;

    #[test]
    fn bitflag_works() {
        let mut flag = 0;
        set_bitflag(&mut flag, 2, true);
        assert_eq!(bitflag(flag, 2), true);
        assert_eq!(bitflag(flag, 4), false);
        set_bitflag(&mut flag, 4, true);
        assert_eq!(bitflag(flag, 2), true);
        assert_eq!(bitflag(flag, 4), true);
        set_bitflag(&mut flag, 2, false);
        assert_eq!(bitflag(flag, 2), false);
        assert_eq!(bitflag(flag, 4), true);
    }

    #[test]
    fn half_bytes_works() {
        assert_eq!(half_bytes(0xAB), (0xA, 0xB));
    }

    #[test]
    fn serialize_works() {
        assert_eq!(serialize(&0x1A2Bi16), &[0x2B, 0x1A]);
        assert_eq!(deserialize::<i16>(&[0x2B, 0x1A]), &0x1A2Bi16);
    }

    #[test]
    #[should_panic]
    fn deserialize_panics_on_src_too_small() {
        deserialize::<u32>(&[0x2B, 0x1A]);
    }

    #[test]
    fn crc32_works() {
        assert_eq!(crc32(b"The quick brown fox jumps over the lazy dog"), 0x414FA339);
    }
}
