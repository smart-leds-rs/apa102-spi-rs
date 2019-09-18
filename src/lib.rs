//! # Use apa102 leds via spi
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! Doesn't use the native brightness settings of the apa102 leds, since that
//! runs at a much lower pwm frequency and thus nerfes the very high color pwm
//! frequency. (According to Adafruit)
//!
//! Needs a type implementing the `blocking::spi::Write` trait.

#![no_std]

use embedded_hal::blocking::spi::Write;
use embedded_hal::spi::{Mode, Phase, Polarity};

use smart_leds_trait::{SmartLedsWrite, RGB8};

/// SPI mode that is needed for this crate
///
/// Provided for convenience
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleHigh,
    phase: Phase::CaptureOnSecondTransition,
};

pub struct Apa102<SPI> {
    spi: SPI,
    postamble_length: u8,
    invert_postamble: bool,
}

impl<SPI, E> Apa102<SPI>
where
    SPI: Write<u8, Error = E>,
{
    pub fn new(spi: SPI) -> Apa102<SPI> {
        Self {
            spi,
            postamble_length: 4,
            invert_postamble: false,
        }
    }

    pub fn new_with_custom_postamble(
        spi: SPI,
        postamble_length: u8,
        invert_postamble: bool,
    ) -> Apa102<SPI> {
        Self {
            spi,
            postamble_length,
            invert_postamble,
        }
    }
}

impl<SPI, E> SmartLedsWrite for Apa102<SPI>
where
    SPI: Write<u8, Error = E>,
{
    type Error = E;
    type Color = RGB8;
    /// Write all the items of an iterator to a apa102 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.spi.write(&[0x00, 0x00, 0x00, 0x00])?;
        for item in iterator {
            let item = item.into();
            self.spi.write(&[0xFF, item.b, item.g, item.r])?;
        }
        for _ in 0..self.postamble_length {
            match self.invert_postamble {
                false => self.spi.write(&[0xFF])?,
                true => self.spi.write(&[0x00])?,
            };
        }
        Ok(())
    }
}
