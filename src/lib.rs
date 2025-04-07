//! Send data to APA102 LEDs via SPI. This crate provides both blocking and asynchronous implementations, which require a HAL crate for your microcontroller with an implementation of the [embedded_hal::spi::SpiBus] or [embedded_hal_async::spi::SpiBus] trait.
//!
//! There are several ways to send pixel data:
//!   * Handle all details of the protocol yourself with the [Apa102Pixel] struct, 8 bit RGB + 5 bits brightness
//!   * Simply provide [smart_leds_trait::RGB8] values, hardcoding maximum brightness. This may be uncomfortably bright.
//!   * Use [FastLED's pseudo-13-bit gamma correction algorithm](https://github.com/FastLED/FastLED/blob/master/APA102.md) to convert [smart_leds_trait::RGB8] + 8 bit brightness to 8 bit RGB + 5 bit brightness.
//!
//! ```
//! # use embedded_hal::spi::{SpiBus, ErrorType, ErrorKind};
//! # struct DummySpi;
//! # impl ErrorType for DummySpi {
//! #   type Error = ErrorKind;
//! # }
//! #
//! # impl SpiBus for DummySpi {
//! #   fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
//! #     Ok(())
//! #   }
//! #
//! #   fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
//! #     Ok(())
//! #   }
//! #
//! #   fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
//! #     Ok(())
//! #   }
//! #
//! #   fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
//! #     Ok(())
//! #   }
//! #
//! #   fn flush(&mut self) -> Result<(), Self::Error> {
//! #     Ok(())
//! #   }
//! # }
//! # let get_spi_peripheral_from_your_hal = DummySpi {};
//! use smart_leds_trait::{SmartLedsWrite, RGB8};
//! use apa102_spi::{Apa102, Apa102Pixel, u5};
//!
//! // You only need to specify MOSI and clock pins for your SPI peripheral.
//! // APA102 LEDs do not send data over MISO and do not have a CS pin.
//! let spi = get_spi_peripheral_from_your_hal;
//! let mut led_strip = Apa102::new(spi);
//!
//! // Specify pixel values as 8 bit RGB + 5 bit brightness
//! let led_buffer = [Apa102Pixel { red: 255, green: 0, blue: 0, brightness: u5::new(1) }];
//! led_strip.write(led_buffer);
//!
//! // Specify pixel values with 8 bit RGB values
//! let led_buffer_rgb = [RGB8 { r: 255, g: 0, b: 0}];
//! // Brightness is set to maximum value (31) in `impl From<RGB8> for Apa102Pixel`
//! led_strip.write(led_buffer_rgb);
//!
//! // Convert RGB8 + 8 bit brightness into Apa102Pixels
//! // using FastLED's pseudo-13-bit gamma correction algorithm.
//! led_strip.write(led_buffer_rgb.map(
//!   |p| Apa102Pixel::from_rgb8_with_brightness(p, 255, None)));
//! ```
//!
//! ## Cargo features
//!   * `defmt`: impl [defmt::Format] for [Apa102Pixel] (off by default)

#![no_std]

mod asynch;
pub use asynch::Apa102Async;

mod pixel;
pub use pixel::Apa102Pixel;
pub use ux::u5;

mod bitshift;
mod math;
mod pseudo13;

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

/// A writer for APA102 LEDs
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
                    0b11100000 | u8::from(item.brightness),
                    item.red,
                    item.green,
                    item.blue,
                ])?,
                PixelOrder::RBG => self.spi.write(&[
                    0b11100000 | u8::from(item.brightness),
                    item.red,
                    item.blue,
                    item.green,
                ])?,
                PixelOrder::GRB => self.spi.write(&[
                    0b11100000 | u8::from(item.brightness),
                    item.green,
                    item.red,
                    item.blue,
                ])?,
                PixelOrder::GBR => self.spi.write(&[
                    0b11100000 | u8::from(item.brightness),
                    item.green,
                    item.blue,
                    item.red,
                ])?,
                PixelOrder::BRG => self.spi.write(&[
                    0b11100000 | u8::from(item.brightness),
                    item.blue,
                    item.red,
                    item.green,
                ])?,
                PixelOrder::BGR => self.spi.write(&[
                    0b11100000 | u8::from(item.brightness),
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
