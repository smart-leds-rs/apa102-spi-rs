//! # Use apa102 leds via spi
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! Needs a type implementing the `blocking::spi::Write` trait.

#![no_std]

mod asynch;
pub use asynch::Apa102Async;

mod pixel;
pub use pixel::Apa102Pixel;

use embedded_hal::spi::SpiBus;
use embedded_hal::spi::{Mode, Phase, Polarity};

use smart_leds_trait::SmartLedsWrite;

/// SPI mode that is needed for this crate
///
/// Provided for convenience
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

pub struct Apa102<SPI> {
    spi: SPI,
    end_frame_length: u8,
    invert_end_frame: bool,
    pixel_order: PixelOrder,
}

/// What order to transmit pixel colors. Different Dotstars
/// need their pixel color data sent in different orders.
pub enum PixelOrder {
    RGB,
    RBG,
    GRB,
    GBR,
    BRG,
    BGR, // Default
}

impl<SPI> Apa102<SPI>
where
    SPI: SpiBus,
{
    /// new constructs a controller for a series of APA102 LEDs.
    /// By default, an End Frame consisting of 32 bits of zeroes is emitted
    /// following the LED data. Control over the size and polarity
    /// of the End Frame is possible using new_with_options().
    /// PixelOrder defaults to BGR ordering, and can also be customized
    /// using new_with_options()
    pub fn new(spi: SPI) -> Apa102<SPI> {
        Self {
            spi,
            end_frame_length: 4,
            invert_end_frame: true,
            pixel_order: PixelOrder::BGR,
        }
    }

    pub fn new_with_options(
        spi: SPI,
        end_frame_length: u8,
        invert_end_frame: bool,
        pixel_order: PixelOrder,
    ) -> Apa102<SPI> {
        Self {
            spi,
            end_frame_length,
            invert_end_frame,
            pixel_order,
        }
    }

    /// Free the owned resources consuming self
    pub fn free(self) -> SPI {
        self.spi
    }
}

impl<SPI> SmartLedsWrite for Apa102<SPI>
where
    SPI: SpiBus,
{
    type Color = Apa102Pixel;
    type Error = SPI::Error;
    /// Write all the items of an iterator to an apa102 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), SPI::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.spi.write(&[0x00, 0x00, 0x00, 0x00])?;
        for item in iterator {
            let item = item.into();
            match self.pixel_order {
                PixelOrder::RGB => self.spi.write(&[
                    0b11100000 | item.brightness,
                    item.red,
                    item.green,
                    item.blue,
                ])?,
                PixelOrder::RBG => self.spi.write(&[
                    0b11100000 | item.brightness,
                    item.red,
                    item.blue,
                    item.green,
                ])?,
                PixelOrder::GRB => self.spi.write(&[
                    0b11100000 | item.brightness,
                    item.green,
                    item.red,
                    item.blue,
                ])?,
                PixelOrder::GBR => self.spi.write(&[
                    0b11100000 | item.brightness,
                    item.green,
                    item.blue,
                    item.red,
                ])?,
                PixelOrder::BRG => self.spi.write(&[
                    0b11100000 | item.brightness,
                    item.blue,
                    item.red,
                    item.green,
                ])?,
                PixelOrder::BGR => self.spi.write(&[
                    0b11100000 | item.brightness,
                    item.blue,
                    item.green,
                    item.red,
                ])?,
            }
        }
        for _ in 0..self.end_frame_length {
            match self.invert_end_frame {
                false => self.spi.write(&[0xFF])?,
                true => self.spi.write(&[0x00])?,
            };
        }
        Ok(())
    }
}
