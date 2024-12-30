// Manually translated to Rust from FastLED's MIT licensed C++ code
// https://github.com/FastLED/FastLED/blob/1c12d96931d8974fba9d64a443a2e7f5850002b2/src/five_bit_hd_gamma.cpp

use crate::{bitshift::*, math::*, Apa102Pixel};
use core::cmp::max;
use smart_leds_trait::{RGB16, RGB8};

pub(crate) fn five_bit_bitshift(mut in_color: RGB16, mut brightness: u8) -> Apa102Pixel {
    if brightness == 0 {
        return Apa102Pixel {
            red: 0,
            green: 0,
            blue: 0,
            brightness: 0,
        };
    }

    if in_color.r == 0 && in_color.g == 0 && in_color.b == 0 {
        return Apa102Pixel {
            red: 0,
            green: 0,
            blue: 0,
            brightness: if brightness <= 0b00011111 {
                brightness
            } else {
                0b00011111
            },
        };
    }

    // Note: One day someone smarter than me will come along and invent a closed
    // form solution for this. However, the numerical method works extremely
    // well and has been optimized to avoid division performance penalties as
    // much as possible.

    // Step 1: Initialize brightness
    let mut v5 = 0b00010000;
    // Step 2: Boost brightness by swapping power with the driver brightness.
    brightness_bitshifter8(&mut v5, &mut brightness, 4);

    // Step 3: Boost brightness of the color channels by swapping power with the
    // driver brightness.
    let mut max_component = max(max(in_color.r, in_color.g), in_color.b);
    let shifts = brightness_bitshifter16(&mut v5, &mut max_component, 4, 2);
    if shifts > 0 {
        in_color.r <<= shifts;
        in_color.g <<= shifts;
        in_color.b <<= shifts;
    }

    // Step 4: scale by final brightness factor.
    if brightness != u8::MAX {
        in_color.r = scale16by8(in_color.r, brightness);
        in_color.g = scale16by8(in_color.g, brightness);
        in_color.b = scale16by8(in_color.b, brightness);
    }

    // brighten hardware brightness by turning on low order bits
    if v5 > 1 {
        // since v5 is a power of two, subtracting one will invert the leading bit
        // and invert all the bits below it.
        // Example: 0b00010000 -1 = 0b00001111
        // So 0b00010000 | 0b00001111 = 0b00011111
        v5 = v5 | (v5 - 1);
    }
    // Step 5: Convert back to 8-bit and output.
    Apa102Pixel {
        red: map16_to_8(in_color.r),
        green: map16_to_8(in_color.g),
        blue: map16_to_8(in_color.b),
        brightness: v5,
    }
}

/// Look up table for gamma16 correction at power of 2.8
#[rustfmt::skip]
static GAMMA_TABLE: [u16; 256] = [
    0,     0,     0,     1,     1,     2,     4,     6,     8,     11,    14,
    18,    23,    29,    35,    41,    49,    57,    67,    77,    88,    99,
    112,   126,   141,   156,   173,   191,   210,   230,   251,   274,   297,
    322,   348,   375,   404,   433,   464,   497,   531,   566,   602,   640,
    680,   721,   763,   807,   853,   899,   948,   998,   1050,  1103,  1158,
    1215,  1273,  1333,  1394,  1458,  1523,  1590,  1658,  1729,  1801,  1875,
    1951,  2029,  2109,  2190,  2274,  2359,  2446,  2536,  2627,  2720,  2816,
    2913,  3012,  3114,  3217,  3323,  3431,  3541,  3653,  3767,  3883,  4001,
    4122,  4245,  4370,  4498,  4627,  4759,  4893,  5030,  5169,  5310,  5453,
    5599,  5747,  5898,  6051,  6206,  6364,  6525,  6688,  6853,  7021,  7191,
    7364,  7539,  7717,  7897,  8080,  8266,  8454,  8645,  8838,  9034,  9233,
    9434,  9638,  9845,  10055, 10267, 10482, 10699, 10920, 11143, 11369, 11598,
    11829, 12064, 12301, 12541, 12784, 13030, 13279, 13530, 13785, 14042, 14303,
    14566, 14832, 15102, 15374, 15649, 15928, 16209, 16493, 16781, 17071, 17365,
    17661, 17961, 18264, 18570, 18879, 19191, 19507, 19825, 20147, 20472, 20800,
    21131, 21466, 21804, 22145, 22489, 22837, 23188, 23542, 23899, 24260, 24625,
    24992, 25363, 25737, 26115, 26496, 26880, 27268, 27659, 28054, 28452, 28854,
    29259, 29667, 30079, 30495, 30914, 31337, 31763, 32192, 32626, 33062, 33503,
    33947, 34394, 34846, 35300, 35759, 36221, 36687, 37156, 37629, 38106, 38586,
    39071, 39558, 40050, 40545, 41045, 41547, 42054, 42565, 43079, 43597, 44119,
    44644, 45174, 45707, 46245, 46786, 47331, 47880, 48432, 48989, 49550, 50114,
    50683, 51255, 51832, 52412, 52996, 53585, 54177, 54773, 55374, 55978, 56587,
    57199, 57816, 58436, 59061, 59690, 60323, 60960, 61601, 62246, 62896, 63549,
    64207, 64869, 65535];

pub(crate) fn five_bit_hd_gamma_bitshift(
    colors: &RGB8,
    brightness: u8,
    color_correction: Option<&RGB8>,
) -> Apa102Pixel {
    if brightness == 0 {
        return Apa102Pixel {
            red: 0,
            blue: 0,
            green: 0,
            brightness: 0,
        };
    }

    let mut rgb16 = RGB16 {
        r: GAMMA_TABLE[colors.r as usize],
        g: GAMMA_TABLE[colors.g as usize],
        b: GAMMA_TABLE[colors.b as usize],
    };

    if let Some(color_correction) = color_correction {
        if color_correction.r != u8::MAX {
            rgb16.r = scale16by8(rgb16.r, color_correction.r);
        }
        if color_correction.g != u8::MAX {
            rgb16.g = scale16by8(rgb16.g, color_correction.g);
        }
        if color_correction.b != u8::MAX {
            rgb16.b = scale16by8(rgb16.b, color_correction.b);
        }
    }

    five_bit_bitshift(rgb16, brightness)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_five_bit_bitshift() {
        #[rustfmt::skip]
        let test_data = [
            (RGB16 {      r:   0, g:     0, b:    0},            0,   // input
            Apa102Pixel { red: 0, green: 0, blue: 0, brightness: 0}), // output

            // 0 brightness brings all colors down to 0
            (RGB16 {      r:   0xffff, g:     0xffff, b:    0xffff},            0,
            Apa102Pixel { red: 0,      green: 0,      blue: 0,      brightness: 0}),

            // color values below 8 become 0 at max brightness
            (RGB16 {      r:   8, g:     7, b:    0},            255,
            Apa102Pixel { red: 1, green: 0, blue: 0, brightness: 1}),

            (RGB16 {      r:   0xffff, g:     0x00f0, b:    0x000f},            0x01,
            Apa102Pixel { red: 0x11,   green: 0x00,   blue: 0x00,   brightness: 0x01}),

            (RGB16 {      r:   0x0100, g:     0x00f0, b:    0x000f},            0xff,
            Apa102Pixel { red: 0x08,   green: 0x08,   blue: 0x00,   brightness: 0x03}),

            (RGB16 {      r:   0x2000, g:     0x1000, b:    0x0f00},            0x20,
            Apa102Pixel { red: 0x20,   green: 0x10,   blue: 0x0f,   brightness: 0x03}),

            (RGB16 {      r:   0xffff, g:     0x8000, b:    0x4000},            0x40,
            Apa102Pixel { red: 0x81,   green: 0x41,   blue: 0x20,   brightness: 0x0f}),

            (RGB16 {      r:   0xffff, g:     0x8000, b:    0x4000},            0x80,
            Apa102Pixel { red: 0x81,   green: 0x41,   blue: 0x20,   brightness: 0x1f}),

            (RGB16 {      r:   0xffff, g:     0xffff, b:    0xffff},            0xff,
            Apa102Pixel { red: 0xff,   green: 0xff,   blue: 0xff,   brightness: 0x1f}),
        ];

        for data in test_data {
            let rgb16 = data.0;
            let result = five_bit_bitshift(rgb16, data.1);
            assert_eq!(result, data.2, "input: {}, brightness: {}", data.0, data.1);
        }
    }

    #[test]
    fn test_five_bit_hd_gamma_bitshift() {
        #[rustfmt::skip]
        let test_data = [
            (RGB8 {       r:   0, g:     0, b:    0},            0,   // input
            Apa102Pixel { red: 0, green: 0, blue: 0, brightness: 0}), // output
            // 0 brightness brings all colors down to 0
            (RGB8 {       r:   255, g:     255, b:    255},          0,
            Apa102Pixel { red: 0,   green: 0,   blue: 0, brightness: 0}),

            (RGB8 {       r:   16, g:     16, b:    16},           16,
            Apa102Pixel { red: 0,  green: 0,  blue: 0, brightness: 1}),

            (RGB8 {       r:   64, g:     64, b:    64},           8,
            Apa102Pixel { red: 4,  green: 4,  blue: 4, brightness: 1}),

            (RGB8 {       r:   255, g:     127, b:    43},           1,
            Apa102Pixel { red: 17,  green: 3,   blue: 0, brightness: 1}),

            (RGB8 {       r:   255, g:     127, b:    43},           64,
            Apa102Pixel { red: 129, green: 21,  blue: 1, brightness: 15}),

            (RGB8 {       r:   255, g:     127, b:    43},           255,
            Apa102Pixel { red: 255, green: 42,  blue: 3, brightness: 31}),

            (RGB8 {       r:   255, g:     255, b:    255},            255,
            Apa102Pixel { red: 255, green: 255, blue: 255, brightness: 31}),
        ];

        for data in test_data {
            let result = five_bit_hd_gamma_bitshift(&data.0, data.1, None);
            assert_eq!(result, data.2, "input {}, brightness {}", data.0, data.1);
        }
    }
}
