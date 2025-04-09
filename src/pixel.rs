use smart_leds_trait::{RGB16, RGB8};
use ux::u5;

/// A single APA102 pixel: 8 bits each for red, green, and blue, plus 5 bits for brightness
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Apa102Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: u5,
}

#[cfg(feature = "defmt")]
impl defmt::Format for Apa102Pixel {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Apa102Pixel {{ red: {=u8:?}, green: {=u8:?}, blue: {=u8:?}, brightness: {=u8:?} }}",
            self.red,
            self.green,
            self.blue,
            u8::from(self.brightness)
        );
    }
}

impl From<RGB8> for Apa102Pixel {
    /// RGB values are copied as-is and brightness is set to the maximum value (31).
    fn from(old: RGB8) -> Self {
        Self {
            red: old.r,
            green: old.g,
            blue: old.b,
            brightness: u5::MAX,
        }
    }
}

impl Apa102Pixel {
    /// Convert an [RGB8] to an [Apa102Pixel] with a specified brightness level.
    /// Any [u8] is a valid brightness level from 0 to 255.
    /// [FastLED's psuedo-13-bit gamma correction algorithm](https://github.com/FastLED/FastLED/blob/master/APA102.md)
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

    /// Convert an [RGB16] to an [Apa102Pixel] with a specified brightness level.
    /// Any [u8] is a valid brightness level from 0 to 255.
    /// [FastLED's psuedo-13-bit gamma correction algorithm](https://github.com/FastLED/FastLED/blob/master/APA102.md)
    /// is used to make use of the dynamic range available from the APA102 protocol, preserving
    /// color detail at low brightness.
    ///
    /// This function does not apply gamma correction; the [RGB16] input is assumed to be gamma corrected already.
    pub fn from_rgb16_with_brightness(rgb16: RGB16, brightness: u8) -> Self {
        crate::pseudo13::five_bit_bitshift(rgb16, brightness)
    }
}
