//! Use APA102 leds via SPI with asynchronous writing of data via the
//! [`embedded_hal_async::spi::SpiBus`](https://docs.rs/embedded-hal-async/latest/embedded_hal_async/spi/trait.SpiBus.html) trait.
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWriteAsync` trait

use crate::{Apa102Pixel, PixelOrder};

use embedded_hal_async::spi::SpiBus;
use smart_leds_trait::SmartLedsWriteAsync;

pub struct Apa102Async<SPI> {
    spi: SPI,
    end_frame_length: u8,
    invert_end_frame: bool,
    pixel_order: PixelOrder,
}

impl<SPI> Apa102Async<SPI>
where
    SPI: SpiBus,
{
    /// new constructs a controller for a series of APA102 LEDs.
    /// By default, an End Frame consisting of 32 bits of zeroes is emitted
    /// following the LED data. Control over the size and polarity
    /// of the End Frame is possible using new_with_options().
    /// PixelOrder defaults to BGR ordering, and can also be customized
    /// using new_with_options()
    pub fn new(spi: SPI) -> Self {
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
    ) -> Self {
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

impl<SPI> SmartLedsWriteAsync for Apa102Async<SPI>
where
    SPI: SpiBus,
{
    type Color = Apa102Pixel;
    type Error = SPI::Error;
    /// Write all the items of an iterator to an apa102 strip
    async fn write<T, I>(&mut self, iterator: T) -> Result<(), SPI::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.spi.write(&[0x00, 0x00, 0x00, 0x00]).await?;
        for item in iterator {
            let item = item.into();
            match self.pixel_order {
                PixelOrder::RGB => {
                    self.spi
                        .write(&[
                            0b11100000 | item.brightness,
                            item.red,
                            item.green,
                            item.blue,
                        ])
                        .await?
                }
                PixelOrder::RBG => {
                    self.spi
                        .write(&[
                            0b11100000 | item.brightness,
                            item.red,
                            item.blue,
                            item.green,
                        ])
                        .await?
                }
                PixelOrder::GRB => {
                    self.spi
                        .write(&[
                            0b11100000 | item.brightness,
                            item.green,
                            item.red,
                            item.blue,
                        ])
                        .await?
                }
                PixelOrder::GBR => {
                    self.spi
                        .write(&[
                            0b11100000 | item.brightness,
                            item.green,
                            item.blue,
                            item.red,
                        ])
                        .await?
                }
                PixelOrder::BRG => {
                    self.spi
                        .write(&[
                            0b11100000 | item.brightness,
                            item.blue,
                            item.red,
                            item.green,
                        ])
                        .await?
                }
                PixelOrder::BGR => {
                    self.spi
                        .write(&[
                            0b11100000 | item.brightness,
                            item.blue,
                            item.green,
                            item.red,
                        ])
                        .await?
                }
            }
        }
        for _ in 0..self.end_frame_length {
            match self.invert_end_frame {
                false => self.spi.write(&[0xFF]).await?,
                true => self.spi.write(&[0x00]).await?,
            };
        }
        Ok(())
    }
}
