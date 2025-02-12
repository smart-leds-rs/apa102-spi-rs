// Manually translated to Rust from FastLED's MIT licensed C++ code

/// Steal brightness from brightness_src and give it to brightness_dst.
/// After this function concludes the multiplication of brightness_dst and brightness_src will remain constant.
pub(crate) fn brightness_bitshifter8(
    brightness_src: &mut u8,
    brightness_dst: &mut u8,
    max_shifts: u8,
) -> u8 {
    if *brightness_dst == 0 || *brightness_src == 0 {
        return 0;
    }
    let mut shifts = 0;
    while shifts < max_shifts && *brightness_src > 1 {
        if *brightness_dst & 0b10000000 > 0 {
            // next shift will overflow
            break;
        }
        *brightness_src >>= 1;
        *brightness_dst <<= 1;
        shifts += 1;
    }
    shifts
}

/// Return value is the number of shifts on the src. Multiply this by the number of steps to get the
/// the number of shifts on the dst.
pub(crate) fn brightness_bitshifter16(
    brightness_src: &mut u8,
    brightness_dst: &mut u16,
    max_shifts: u8,
    steps: u8,
) -> u8 {
    if *brightness_dst == 0 || *brightness_src == 0 {
        return 0;
    }
    let mut overflow_mask = 0b1000000000000000;
    let mut i = 1;
    while i < steps {
        overflow_mask >>= 1;
        overflow_mask |= 0b1000000000000000;
        i += 1;
    }
    let underflow_mask = 0x1;

    // Steal brightness from brightness_src and give it to brightness_dst.
    // After this function concludes the multiplication of brightness_dst and brightness_src will remain
    // constant.
    let mut shifts = 0;
    while shifts < max_shifts {
        if *brightness_src & underflow_mask > 0 {
            break;
        }
        if *brightness_dst & overflow_mask > 0 {
            // next shift will overflow
            break;
        }
        *brightness_src >>= 1;
        *brightness_dst <<= steps;
        shifts += 1;
    }
    shifts
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_brightness_bitshifter8_random() {
        let mut count = 0;
        for _ in 0..10000 {
            let mut brightness_src = 0b10000000 >> fastrand::u8(..) % 6;
            let mut brightness_dst = fastrand::u8(..);
            let product = brightness_src as u16 * brightness_dst as u16;
            let shifts = brightness_bitshifter8(&mut brightness_src, &mut brightness_dst, 7);
            let new_product = brightness_src as u16 * brightness_dst as u16;
            assert_eq!(product, new_product);
            if shifts > 0 {
                count += 1;
            }
        }
        assert!(count > 0);
    }

    #[test]
    fn test_brightness_bitshifter8_fixed_data() {
        #[rustfmt::skip]
        let test_data = [
            // brightness_bitshifter8 is always called with brightness_src = 0b00010000
            [ // test case
                // src       dst
                [0b00010000, 0b00000000], // input
                [0b00010000, 0b00000000], // output
            ],
            [
                [0b00010000, 0b00000001],
                [0b00000001, 0b00010000]
            ],
            [
                [0b00010000, 0b00000100],
                [0b00000001, 0b01000000]
            ],
            [
                [0b00010000, 0b00010000],
                [0b00000010, 0b10000000]
            ],
            [
                [0b00010000, 0b00001010],
                [0b00000001, 0b10100000]
            ],
            [
                [0b00010000, 0b00101010],
                [0b00000100, 0b10101000]
            ],
            [
                [0b00010000, 0b11101010],
                [0b00010000, 0b11101010]
            ],
        ];

        for data in test_data {
            let mut brightness_src = data[0][0];
            let mut brightness_dst = data[0][1];
            let shifts = brightness_bitshifter8(&mut brightness_src, &mut brightness_dst, 4);
            assert_eq!(
                brightness_src, data[1][0],
                "
input  brightness_src: {} ; input  brightness_dst: {}
output brightness_src: {brightness_src} ; output brightness_dst: {brightness_dst}
shifts: {shifts}",
                data[0][0], data[0][1]
            );
            assert_eq!(
                brightness_dst, data[1][1],
                "
input  brightness_src: {} ; input  brightness_dst: {}
output brightness_src: {brightness_src} ; output brightness_dst: {brightness_dst}
shifts: {shifts}",
                data[0][0], data[0][1]
            );
        }
    }

    #[test]
    fn test_brightness_bitshifter16_steps2() {
        let mut brightness_src = 0x1 << 1;
        let mut brightness_dst = 0x1 << 2;
        let max_shifts = 8;

        let shifts =
            brightness_bitshifter16(&mut brightness_src, &mut brightness_dst, max_shifts, 2);

        assert_eq!(shifts, 1);
        assert_eq!(brightness_src, 1);
        assert_eq!(brightness_dst, 0x1 << 4);
    }

    #[test]
    fn test_brightness_bitshifter16_steps1() {
        let mut brightness_src = 0x1 << 1;
        let mut brightness_dst = 0x1 << 1;
        let max_shifts = 8;

        let shifts =
            brightness_bitshifter16(&mut brightness_src, &mut brightness_dst, max_shifts, 1);

        assert_eq!(shifts, 1);
        assert_eq!(brightness_src, 1);
        assert_eq!(brightness_dst, 0x1 << 2);
    }

    #[test]
    fn test_brightness_bitshifter16_random() {
        let mut count = 0;
        for _ in 0..10000 {
            let mut brightness_src = 0b10000000 >> (fastrand::u8(..) % 8);
            let mut brightness_dst = fastrand::u16(..);
            let product = (brightness_src as u32 >> 8) * brightness_dst as u32;
            let max_shifts = 8;
            let steps = 2;

            let shifts = brightness_bitshifter16(
                &mut brightness_src,
                &mut brightness_dst,
                max_shifts,
                steps,
            );

            let new_product = (brightness_src as u32 >> 8) * brightness_dst as u32;
            assert_eq!(product, new_product);
            if shifts > 0 {
                count += 1;
            }
        }
        assert!(count > 0);
    }

    #[test]
    fn test_brightness_bitshifter16_fixed_data() {
        let test_data = [
            // brightness_bitshifter16 is always called with brightness_src between 0b00000001 - 0b00010000
            [
                // test case
                // src       dst
                [0b00000001, 0b0000000000000000], // input
                [0b00000001, 0b0000000000000000], // output
            ],
            [
                [0b00000001, 0b0000000000000001],
                [0b00000001, 0b0000000000000001],
            ],
            [
                [0b00000001, 0b0000000000000010],
                [0b00000001, 0b0000000000000010],
            ],
            [
                [0b00000010, 0b0000000000000001],
                [0b00000001, 0b0000000000000100],
            ],
            [
                [0b00001010, 0b0000000000001010],
                [0b00000101, 0b0000000000101000],
            ],
            [
                [0b00010000, 0b0000111000100100],
                [0b00000100, 0b1110001001000000],
            ],
            [
                [0b00010000, 0b0011100010010010],
                [0b00001000, 0b1110001001001000],
            ],
            [
                [0b00010000, 0b0110001001001110],
                [0b00010000, 0b0110001001001110],
            ],
            [
                [0b00010000, 0b1110001001001110],
                [0b00010000, 0b1110001001001110],
            ],
        ];

        for data in test_data {
            let mut brightness_src = data[0][0] as u8;
            let mut brightness_dst = data[0][1];
            let shifts = brightness_bitshifter16(&mut brightness_src, &mut brightness_dst, 4, 2);
            assert_eq!(
                brightness_src, data[1][0] as u8,
                "
input  brightness_src: {} ; input  brightness_dst: {}
output brightness_src: {brightness_src} ; output brightness_dst: {brightness_dst}
shifts (by 2 bits) : {shifts}",
                data[0][0], data[0][1]
            );
            assert_eq!(
                brightness_dst, data[1][1],
                "
input  brightness_src: {} ; input  brightness_dst: {}
output brightness_src: {brightness_src} ; output brightness_dst: {brightness_dst}
shifts (by 2 bits) : {shifts}",
                data[0][0], data[0][1]
            );
        }
    }
}
