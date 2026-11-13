//! LEB128 writing and zigzag mapping. Reading lives on the cursor.

/// Longest encoding of a 64-bit varint.
const MAX_VARINT_BYTES: usize = 10;

/// Appends `value` as a minimal unsigned LEB128 varint.
pub(super) fn write_varint(out: &mut Vec<u8>, value: u64) {
    let (bytes, used) = varint_bytes(value);
    out.extend(bytes.iter().take(used));
}

/// Encodes `value` into a fixed buffer and reports how many bytes it used.
pub(super) fn varint_bytes(mut value: u64) -> ([u8; MAX_VARINT_BYTES], usize) {
    let mut bytes = [0u8; MAX_VARINT_BYTES];
    let mut used = 0usize;
    for slot in &mut bytes {
        used = used.saturating_add(1);
        let low = low_seven_bits(value);
        value /= 128;
        if value == 0 {
            *slot = low;
            break;
        }
        *slot = low | 0x80;
    }
    (bytes, used)
}

/// The lowest seven bits of `value`.
fn low_seven_bits(value: u64) -> u8 {
    let [first, ..] = value.to_le_bytes();
    first & 0x7f
}

/// Appends a length or count as a varint.
pub(super) fn write_len(out: &mut Vec<u8>, len: usize) {
    write_varint(out, u64::try_from(len).unwrap_or(u64::MAX));
}

/// Maps a signed integer to an unsigned one with small magnitudes first.
pub(super) fn zigzag(value: i64) -> u64 {
    let bits = u64::from_ne_bytes(value.to_ne_bytes());
    let sign = if value < 0 { u64::MAX } else { 0 };
    bits.wrapping_shl(1) ^ sign
}

/// Inverse of [`zigzag`].
pub(super) fn unzigzag(value: u64) -> i64 {
    let half = value / 2;
    let bits = if value & 1 == 1 { !half } else { half };
    i64::from_ne_bytes(bits.to_ne_bytes())
}
