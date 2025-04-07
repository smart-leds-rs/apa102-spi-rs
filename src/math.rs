// Manually translated to Rust from FastLED's MIT licensed C++ code

/// Scale a 16-bit unsigned value by an 8-bit value, which is treated
/// as the numerator of a fraction whose denominator is `u8::MAX`.
///
/// In other words, it computes `i * (scale / u8::MAX)`
pub(crate) fn scale16by8(i: u16, scale: u8) -> u16 {
    if scale == 0 {
        return 0;
    }
    ((i as u32 * (1 + scale as u32)) >> 8) as u16
}

/// Maps an integer from one integer size to another.
///
/// For example, a value representing 40% as a `u16` would be `26,214 / 65,535`.
/// Converting that to a `u8` would return `102 / 255`, exactly 40% through the
/// range of values representable by a `u8`.
pub(crate) fn map16_to_8(x: u16) -> u8 {
    // Tested to be nearly identical to double precision floating point
    // doing this operation.
    if x >= 0xff00 {
        return 0xff;
    }
    ((x + 128) >> 8) as u8
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    // This test was translated from FastLED's test
    fn scale16by8_test() {
        assert_eq!(scale16by8(0, 0), 0);
        assert_eq!(scale16by8(0, 1), 0);
        assert_eq!(scale16by8(1, 0), 0);
        assert_eq!(scale16by8(0xffff, 0xff), 0xffff);
        assert_eq!(scale16by8(0xffff, 0xff >> 1), 0xffff >> 1);
        assert_eq!(scale16by8(0xffff >> 1, 0xff >> 1), 0xffff >> 2);

        let mut i = 0;
        while i < 16 {
            let mut j = 0;
            while j < 8 {
                let total_bitshift = i + j;
                if total_bitshift > 7 {
                    break;
                }
                assert_eq!(scale16by8(0xffff >> i, 0xff >> j), 0xffff >> total_bitshift);
                j += 1;
            }
            i += 1;
        }
    }

    #[test]
    fn map16_to_8_test() {
        assert_eq!(map16_to_8(u16::MAX), u8::MAX);
        assert_eq!(map16_to_8(49151), 192); // 75% of range
        assert_eq!(map16_to_8(26214), 102); // 40% of range
        assert_eq!(map16_to_8(16383), 64); // 25% of range
        assert_eq!(map16_to_8(0), 0);
    }
}
