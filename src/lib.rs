//! # Use apa102 leds via spi
//!
//!

#![no_std]

extern crate embedded_hal as hal;

use hal::spi::{FullDuplex, Mode, Phase, Polarity};

use smart_leds_trait::{Color, SmartLedsWrite};

use nb;
use nb::block;

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
    SPI: FullDuplex<u8, Error = E>,
{
    pub fn new(spi: SPI) -> Apa102<SPI> {
        Self { spi }
    }
}

impl<SPI, E> SmartLedsWrite for Apa102<SPI>
where
    SPI: FullDuplex<u8, Error = E>,
{
    type Error = E;
    /// Write all the items of an iterator to a apa102 strip
    fn write<T>(&mut self, iterator: T) -> Result<(), E>
    where
        T: Iterator<Item = Color>,
    {
        for _ in 0..4 {
            block!(self.spi.send(0))?;
            self.spi.read().ok();
        }
        for item in iterator {
            block!(self.spi.send(0xFF))?;
            self.spi.read().ok();
            block!(self.spi.send(item.b))?;
            self.spi.read().ok();
            block!(self.spi.send(item.g))?;
            self.spi.read().ok();
            block!(self.spi.send(item.r))?;
            self.spi.read().ok();
        }
        for _ in 0..4 {
            block!(self.spi.send(0xFF))?;
            self.spi.read().ok();
        }
        Ok(())
    }
}
