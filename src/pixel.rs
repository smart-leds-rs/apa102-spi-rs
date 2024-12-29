use smart_leds_trait::RGB8;

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
