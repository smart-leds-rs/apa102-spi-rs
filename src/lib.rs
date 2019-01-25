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

use smart_leds_trait::{Color, SmartLedsWrite};

/// SPI mode that is needed for this crate
///
/// Provided for convenience
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleHigh,
    phase: Phase::CaptureOnSecondTransition,
};

pub struct Apa102<SPI> {
    spi: SPI,
}

impl<SPI, E> Apa102<SPI>
where
    SPI: Write<u8, Error = E>,
{
    pub fn new(spi: SPI) -> Apa102<SPI> {
        Self { spi }
    }
}

impl<SPI, E> SmartLedsWrite for Apa102<SPI>
where
    SPI: Write<u8, Error = E>,
{
    type Error = E;
    /// Write all the items of an iterator to a apa102 strip
    fn write<T>(&mut self, iterator: T) -> Result<(), E>
    where
        T: Iterator<Item = Color>,
    {
        self.spi.write(&[0x00, 0x00, 0x00, 0x00])?;
        for item in iterator {
            self.spi.write(&[0xFF, item.b, item.g, item.r])?;
        }
        self.spi.write(&[0xFF, 0xFF, 0xFF, 0xFF])?;
        Ok(())
    }
}
