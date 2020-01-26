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
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

pub struct Apa102<SPI> {
    spi: SPI,
    end_frame_length: u8,
    invert_end_frame: bool,
    pixel_type: PixelType,
}

/// What order to transmit pixel colors. Different Dotstars
/// need their pixel color data sent in different orders.
pub enum PixelType {
    RGB,
    RBG,
    GRB,
    GBR,
    BRG,
    BGR, // Default
}

impl<SPI, E> Apa102<SPI>
where
    SPI: Write<u8, Error = E>,
{
    /// new constructs a controller for a series of APA102 LEDs.
    /// By default, an End Frame consisting of 32 bits of zeroes is emitted
    /// following the LED data. Control over the size and polarity
    /// of the End Frame is possible using new_with_custom_postamble().
    pub fn new(spi: SPI) -> Apa102<SPI> {
        Self {
            spi,
            end_frame_length: 4,
            invert_end_frame: true,
            pixel_type: PixelType::BGR,
        }
    }

    pub fn new_with_custom_postamble(
        spi: SPI,
        end_frame_length: u8,
        invert_end_frame: bool,
        pixel_type: PixelType,
    ) -> Apa102<SPI> {
        Self {
            spi,
            end_frame_length,
            invert_end_frame,
            pixel_type,
        }
    }
}

impl<SPI, E> SmartLedsWrite for Apa102<SPI>
where
    SPI: Write<u8, Error = E>,
{
    type Error = E;
    type Color = RGB8;
    /// Write all the items of an iterator to an apa102 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.spi.write(&[0x00, 0x00, 0x00, 0x00])?;
        for item in iterator {
            let item = item.into();
            match self.pixel_type {
                PixelType::RGB => self.spi.write(&[0xFF, item.r, item.g, item.b])?,
                PixelType::RBG => self.spi.write(&[0xFF, item.r, item.b, item.g])?,
                PixelType::GRB => self.spi.write(&[0xFF, item.g, item.r, item.b])?,
                PixelType::GBR => self.spi.write(&[0xFF, item.g, item.b, item.r])?,
                PixelType::BRG => self.spi.write(&[0xFF, item.b, item.r, item.g])?,
                PixelType::BGR => self.spi.write(&[0xFF, item.b, item.g, item.r])?,
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
