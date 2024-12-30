use smart_leds_trait::{RGB16, RGB8};

/// This struct represents a single APA102 pixel, which uses 8 bits each for red, green, and blue, plus 5 bits for brightness.
/// Brightness is represented by a `u8` without any checks for valid values to make this struct a
/// zero-cost abstraction. Any `u8` values above the maximum for 5 bits (0b00011111 in binary or 31 in decimal)
/// will be truncated to the maximum when writing data to the APA102 LEDs.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Apa102Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: u8,
}

impl From<RGB8> for Apa102Pixel {
    /// RGB values are copied as-is and brightness is set to the maximum value (31).
    fn from(old: RGB8) -> Self {
        Self {
            red: old.r,
            green: old.g,
            blue: old.b,
            brightness: 0b00011111,
        }
    }
}

impl Apa102Pixel {
    /// Convert an [rgb::RGB8](https://docs.rs/rgb/latest/rgb/type.RGB8.html) to an [Apa102Pixel]
    /// with a specified brightness level. Any [u8] is a valid brightness level from 0 to 255.
    /// [FastLED's psuedo-13-bit gamma correction algorithm](https://github.com/FastLED/FastLED/blob/d5aaf65be19782f3e52b8b0fe38778f14376a293/APA102.md)
    /// is used to make use of the dynamic range available from the APA102 protocol, preserving
    /// color detail at low brightness. In short, it converts:
    ///
    /// RGB8 + 8-bit brightness → RGB16 + 5-bit gamma → RGB8 + 5-bit gamma
    ///
    /// Optional color correction can be applied between the gamma correction and bitshifting steps.
    pub fn from_rgb8_with_brightness(
        rgb8: RGB8,
        brightness: u8,
        color_correction: Option<&RGB8>,
    ) -> Self {
        crate::pseudo13::five_bit_hd_gamma_bitshift(&rgb8, brightness, color_correction)
    }

    /// Convert an [rgb::RGB16](https://docs.rs/rgb/latest/rgb/type.RGB16.html) to an [Apa102Pixel]
    /// with a specified brightness level. Any [u8] is a valid brightness level from 0 to 255.
    /// [FastLED's psuedo-13-bit gamma correction algorithm](https://github.com/FastLED/FastLED/blob/d5aaf65be19782f3e52b8b0fe38778f14376a293/APA102.md)
    /// is used to make use of the dynamic range available from the APA102 protocol, preserving
    /// color detail at low brightness.
    ///
    /// This function does not apply gamma correction; the [rgb::RGB16](https://docs.rs/rgb/latest/rgb/type.RGB16.html)
    /// input is assumed to be gamma corrected already.
    pub fn from_rgb16_with_brightness(rgb16: RGB16, brightness: u8) -> Self {
        crate::pseudo13::five_bit_bitshift(rgb16, brightness)
    }
}
