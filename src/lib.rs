//! Send data to APA102 LEDs (also known as DotStar LEDs) via SPI. This crate is also compatible with SK9822 LEDs which share the same protocol as APA102 LEDs. Both blocking and asynchronous implementations are provided, which require a HAL crate for your microcontroller with an implementation of the [embedded_hal::spi::SpiBus] or [embedded_hal_async::spi::SpiBus] trait.
//!
//! There are several ways to send pixel data:
//!   * Handle all details of the protocol yourself with the [Apa102Pixel] struct, 8 bit RGB + 5 bits brightness
//!   * Simply provide [RGB8] values, hardcoding maximum brightness. This may be uncomfortably bright.
//!   * Use [FastLED's pseudo-13-bit gamma correction algorithm](https://github.com/FastLED/FastLED/blob/master/APA102.md) to convert [RGB8] + 8 bit brightness to 8 bit RGB + 5 bit brightness.
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
//! use apa102_spi::{u5, Apa102Pixel, Apa102Writer, PixelOrder, SmartLedsWrite, RGB8};
//!
//! // You only need to specify MOSI and clock pins for your SPI peripheral.
//! // APA102 LEDs do not send data over MISO and do not have a CS pin.
//! let spi = get_spi_peripheral_from_your_hal;
//! let mut led_strip = Apa102Writer::new(spi, 1, PixelOrder::default());
//!
//! // Specify pixel values as 8 bit RGB + 5 bit brightness
//! let led_buffer = [Apa102Pixel { red: 255, green: 0, blue: 0, brightness: u5::new(1) }];
//! led_strip.write(led_buffer);
//!
//! // Specify pixel values with 8 bit RGB values
//! let led_buffer_rgb = [RGB8 { r: 255, g: 0, b: 0 }];
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

pub use smart_leds_trait::{SmartLedsWrite, SmartLedsWriteAsync, RGB, RGB16, RGB8};

mod pixel;
pub use pixel::Apa102Pixel;
pub use ux::u5;

mod bitshift;
mod math;
mod pseudo13;

use embedded_hal::spi::{Mode, Phase, Polarity};

/// SPI mode that is needed for this crate
///
/// Provided for convenience
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// What order to transmit pixel colors. The standard order
/// is [PixelOrder::BGR], however in practice, some LEDs
/// swap the order of the colors in the protocol.
pub enum PixelOrder {
    RGB,
    RBG,
    GRB,
    GBR,
    BRG,
    BGR,
}

impl Default for PixelOrder {
    fn default() -> Self {
        PixelOrder::BGR
    }
}

#[path = "."]
mod asynchronous {
    use bisync::asynchronous::*;
    use embedded_hal_async::spi::SpiBus;
    use smart_leds_trait::SmartLedsWriteAsync as SmartLedsWrite;
    mod writer;
    pub use writer::*;
}
pub use asynchronous::Apa102Writer as Apa102WriterAsync;

#[path = "."]
mod blocking {
    use bisync::synchronous::*;
    use embedded_hal::spi::SpiBus;
    use smart_leds_trait::SmartLedsWrite;
    #[allow(clippy::duplicate_mod)]
    mod writer;
    pub use writer::*;
}
pub use blocking::Apa102Writer;
